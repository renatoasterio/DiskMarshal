//! State and logic of the network screen: the live traffic, the analysis of
//! the connection, the speed test, and the changes the user approves.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use diskmarshal_application::ports::{
    InspectionError, PrivilegeError, SpeedTestError, TrafficProbeError,
};
use diskmarshal_application::use_cases::{AnalyzeNetwork, MeasureTraffic, TuneNetwork};
use diskmarshal_domain::network::{
    Finding, NetworkChange, NetworkReport, Optimization, ProgramTraffic, SpeedTest, TrafficMeter,
    TrafficSnapshot,
};
use egui::Context;

use crate::app::UiServices;
use crate::i18n::pt_br;
use crate::tasks::{Feed, Task, TaskPoll};

/// Time between readings of the traffic. Each reading starts a program, so
/// reading more often would cost more processor time than it is worth.
const SAMPLE_INTERVAL: Duration = Duration::from_secs(2);
/// Readings kept for the traces: two minutes.
const HISTORY_LENGTH: usize = 60;
/// Readings taken, and the time between them, when the screen is filled in
/// before the first frame.
const PRELOAD_READINGS: usize = 24;
const PRELOAD_INTERVAL: Duration = Duration::from_millis(120);
/// The top of the plot is never lower than this, in megabits per second, so
/// an idle connection draws a flat line instead of magnified noise.
const LOWEST_PLOT_TOP: f32 = 10.0;
/// Room left above the highest reading of the plot.
const PLOT_HEADROOM: f32 = 1.2;

/// The parts of the screen.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum NetworkTab {
    /// Who is using the network right now.
    #[default]
    Traffic,
    /// How good the connection is.
    Analysis,
    /// What can be changed to make it faster.
    Tuning,
}

impl NetworkTab {
    pub(crate) const ALL: [Self; 3] = [Self::Traffic, Self::Analysis, Self::Tuning];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Traffic => pt_br::NETWORK_TAB_TRAFFIC,
            Self::Analysis => pt_br::NETWORK_TAB_ANALYSIS,
            Self::Tuning => pt_br::NETWORK_TAB_TUNING,
        }
    }

    /// Finds a tab by its identifier, stable across languages.
    fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "traffic" => Some(Self::Traffic),
            "analysis" => Some(Self::Analysis),
            "tuning" => Some(Self::Tuning),
            _ => None,
        }
    }
}

/// What a request to change settings is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChangeKind {
    /// Apply the selected optimizations.
    Apply,
    /// Take back what was applied.
    Undo,
}

/// Where a request to change settings stands.
pub(crate) enum ChangeFlow {
    /// Nothing was asked.
    Closed,
    /// The user is reading what will change.
    Confirming(ChangeKind),
    /// macOS is authenticating the user or the changes are being applied.
    Running {
        kind: ChangeKind,
        changes: Vec<NetworkChange>,
        /// The optimizations the changes belong to, by position.
        indexes: Vec<usize>,
        task: Task<Result<(), PrivilegeError>>,
    },
    /// The changes were applied.
    Done(ChangeKind, Vec<NetworkChange>),
    /// The changes were not applied; the text says why.
    Failed(String),
}

/// The traces of the plot, each reading as a share of `top`.
pub(crate) struct PlotData {
    pub download: Vec<f32>,
    pub upload: Vec<f32>,
    /// What the top of the plot stands for, in megabits per second.
    pub top: f32,
}

/// Everything the network screen remembers between frames.
pub(crate) struct NetworkViewModel {
    pub tab: NetworkTab,
    feed: Option<Feed<Result<TrafficSnapshot, TrafficProbeError>>>,
    /// Kept while the readings are paused, so the totals carry on.
    meter: Arc<Mutex<TrafficMeter>>,
    traffic: Option<TrafficSnapshot>,
    traffic_failure: Option<String>,
    /// Speeds over time, in megabits per second, oldest first.
    download_history: VecDeque<f32>,
    upload_history: VecDeque<f32>,
    selected_program: Option<String>,
    analysis: Option<Task<Result<NetworkReport, InspectionError>>>,
    report: Option<NetworkReport>,
    analysis_failure: Option<String>,
    speed: Option<Task<Result<SpeedTest, SpeedTestError>>>,
    speed_test: Option<SpeedTest>,
    speed_failure: Option<String>,
    optimizations: Vec<Optimization>,
    selected: Vec<bool>,
    applied: Vec<bool>,
    /// The changes that take back what was applied, the latest first.
    undo: Vec<NetworkChange>,
    notice: Option<&'static str>,
    pub flow: ChangeFlow,
}

impl Default for NetworkViewModel {
    fn default() -> Self {
        Self {
            tab: NetworkTab::default(),
            feed: None,
            meter: Arc::default(),
            traffic: None,
            traffic_failure: None,
            download_history: VecDeque::new(),
            upload_history: VecDeque::new(),
            selected_program: None,
            analysis: None,
            report: None,
            analysis_failure: None,
            speed: None,
            speed_test: None,
            speed_failure: None,
            optimizations: Vec::new(),
            selected: Vec::new(),
            applied: Vec::new(),
            undo: Vec::new(),
            notice: None,
            flow: ChangeFlow::Closed,
        }
    }
}

impl NetworkViewModel {
    /// Starts reading the traffic in the background, when it is not being
    /// read already.
    pub(crate) fn ensure_started(&mut self, ctx: &Context, services: &UiServices) {
        if self.feed.is_none() {
            let (probe, meter) = (services.traffic_probe.clone(), Arc::clone(&self.meter));
            self.feed = Some(Feed::spawn(ctx, SAMPLE_INTERVAL, move || {
                let mut meter = meter.lock().unwrap_or_else(PoisonError::into_inner);
                MeasureTraffic::new(probe.as_ref()).execute(&mut meter)
            }));
        }
    }

    /// Stops reading the traffic. The totals carry on from where they
    /// were when the readings start again.
    pub(crate) fn pause(&mut self) {
        self.feed = None;
    }

    /// Reads the traffic a few times and analyzes the connection right now,
    /// blocking until it is done. Used to open the app with the screen
    /// already filled in. The speed test spends real data, so it runs only
    /// when `with_speed_test` says so.
    pub(crate) fn load_now(
        &mut self,
        services: &UiServices,
        tab: Option<&str>,
        with_speed_test: bool,
    ) {
        if let Some(tab) = tab.and_then(NetworkTab::from_slug) {
            self.tab = tab;
        }
        let measure = MeasureTraffic::new(services.traffic_probe.as_ref());
        for _ in 0..PRELOAD_READINGS {
            let reading = {
                let mut meter = self.meter.lock().unwrap_or_else(PoisonError::into_inner);
                measure.execute(&mut meter)
            };
            self.record(reading);
            thread::sleep(PRELOAD_INTERVAL);
        }
        let analysis = AnalyzeNetwork::new(
            services.connection_inspector.as_ref(),
            services.reachability.as_ref(),
        )
        .execute();
        self.accept_analysis(analysis, services);
        if with_speed_test {
            self.accept_speed_test(services.speed_tester.measure());
        }
    }

    /// Picks up what arrived since the last frame: readings, the analysis,
    /// the speed test and the answer to a change.
    pub(crate) fn poll(&mut self, services: &UiServices) {
        let arrived = self.feed.as_ref().map(Feed::drain).unwrap_or_default();
        for reading in arrived {
            self.record(reading);
        }
        if let Some(outcome) = self.analysis.as_ref().and_then(finished) {
            self.analysis = None;
            match outcome {
                Some(analysis) => self.accept_analysis(analysis, services),
                None => self.analysis_failure = Some(pt_br::NETWORK_TASK_LOST.to_owned()),
            }
        }
        if let Some(outcome) = self.speed.as_ref().and_then(finished) {
            self.speed = None;
            match outcome {
                Some(test) => self.accept_speed_test(test),
                None => self.speed_failure = Some(pt_br::NETWORK_TASK_LOST.to_owned()),
            }
        }
        self.poll_change();
    }

    fn record(&mut self, reading: Result<TrafficSnapshot, TrafficProbeError>) {
        match reading {
            Ok(snapshot) => {
                // The first reading only sets the starting point of the
                // counters: it has no speed to draw.
                let has_speed = self.traffic.is_some();
                for (history, rate) in [
                    (&mut self.download_history, snapshot.download),
                    (&mut self.upload_history, snapshot.upload),
                ] {
                    if !has_speed {
                        continue;
                    }
                    if history.len() == HISTORY_LENGTH {
                        history.pop_front();
                    }
                    history.push_back(rate.megabits_per_second() as f32);
                }
                self.traffic = Some(snapshot);
                self.traffic_failure = None;
            }
            Err(error) => self.traffic_failure = Some(pt_br::traffic_probe_error(&error)),
        }
    }

    /// The latest reading of the traffic.
    pub(crate) fn traffic(&self) -> Option<&TrafficSnapshot> {
        self.traffic.as_ref()
    }

    /// Why the traffic could not be read, when the last reading failed.
    pub(crate) fn traffic_failure(&self) -> Option<&str> {
        self.traffic_failure.as_deref()
    }

    /// The traces of the plot, on a scale that follows the highest reading.
    pub(crate) fn plot(&self) -> PlotData {
        let highest = self
            .download_history
            .iter()
            .chain(&self.upload_history)
            .fold(0.0_f32, |highest, rate| highest.max(*rate));
        let top = ((highest * PLOT_HEADROOM).max(LOWEST_PLOT_TOP) / LOWEST_PLOT_TOP).ceil()
            * LOWEST_PLOT_TOP;
        let scaled = |history: &VecDeque<f32>| history.iter().map(|rate| rate / top).collect();
        PlotData {
            download: scaled(&self.download_history),
            upload: scaled(&self.upload_history),
            top,
        }
    }

    /// The program whose connections are listed: the one the user picked
    /// while it has traffic, the busiest one otherwise.
    pub(crate) fn shown_program(&self) -> Option<&ProgramTraffic> {
        let programs = &self.traffic.as_ref()?.programs;
        self.selected_program
            .as_ref()
            .and_then(|picked| programs.iter().find(|program| &program.name == picked))
            .or_else(|| programs.first())
    }

    /// Lists the connections of `program` from now on.
    pub(crate) fn select_program(&mut self, program: &str) {
        self.selected_program = Some(program.to_owned());
    }

    /// Analyzes the connection in the background.
    pub(crate) fn analyze(&mut self, ctx: &Context, services: &UiServices) {
        if self.analysis.is_some() {
            return;
        }
        self.analysis_failure = None;
        let services = services.clone();
        self.analysis = Some(Task::spawn(ctx, move |_, _| {
            AnalyzeNetwork::new(
                services.connection_inspector.as_ref(),
                services.reachability.as_ref(),
            )
            .execute()
        }));
    }

    fn accept_analysis(
        &mut self,
        analysis: Result<NetworkReport, InspectionError>,
        services: &UiServices,
    ) {
        match analysis {
            Ok(report) => {
                let can_change = services.network_tuner.is_available();
                self.optimizations = report.optimizations();
                self.selected = self
                    .optimizations
                    .iter()
                    .map(|offer| can_change && offer.is_recommended)
                    .collect();
                self.applied = vec![false; self.optimizations.len()];
                self.notice = None;
                self.analysis_failure = None;
                self.report = Some(report);
            }
            Err(error) => self.analysis_failure = Some(pt_br::inspection_error(&error)),
        }
    }

    /// Whether the connection is being analyzed right now.
    pub(crate) fn is_analyzing(&self) -> bool {
        self.analysis.is_some()
    }

    /// What the last analysis found.
    pub(crate) fn report(&self) -> Option<&NetworkReport> {
        self.report.as_ref()
    }

    /// Why the last analysis produced nothing, when it failed.
    pub(crate) fn analysis_failure(&self) -> Option<&str> {
        self.analysis_failure.as_deref()
    }

    /// Everything concluded so far, what asks for attention first: from the
    /// speed test, from the analysis and from the traffic right now.
    pub(crate) fn findings(&self) -> Vec<Finding> {
        let mut findings: Vec<Finding> = self
            .speed_test
            .as_ref()
            .and_then(Finding::of_speed_test)
            .into_iter()
            .chain(self.report.iter().flat_map(NetworkReport::findings))
            .chain(self.traffic.as_ref().and_then(Finding::of_traffic))
            .collect();
        findings.sort_by_key(Finding::severity);
        findings
    }

    /// Runs the speed test in the background.
    pub(crate) fn measure_speed(&mut self, ctx: &Context, services: &UiServices) {
        if self.speed.is_some() {
            return;
        }
        self.speed_failure = None;
        let tester = services.speed_tester.clone();
        self.speed = Some(Task::spawn(ctx, move |_, _| tester.measure()));
    }

    fn accept_speed_test(&mut self, test: Result<SpeedTest, SpeedTestError>) {
        match test {
            Ok(test) => {
                self.speed_test = Some(test);
                self.speed_failure = None;
            }
            Err(error) => self.speed_failure = Some(pt_br::speed_test_error(&error)),
        }
    }

    /// Whether the speed test is running right now.
    pub(crate) fn is_measuring_speed(&self) -> bool {
        self.speed.is_some()
    }

    /// What the last speed test found.
    pub(crate) fn speed_test(&self) -> Option<&SpeedTest> {
        self.speed_test.as_ref()
    }

    /// Why the last speed test produced nothing, when it failed.
    pub(crate) fn speed_failure(&self) -> Option<&str> {
        self.speed_failure.as_deref()
    }

    /// The changes offered for this connection.
    pub(crate) fn optimizations(&self) -> &[Optimization] {
        &self.optimizations
    }

    pub(crate) fn is_selected(&self, index: usize) -> bool {
        self.selected.get(index).copied().unwrap_or(false)
    }

    pub(crate) fn is_applied(&self, index: usize) -> bool {
        self.applied.get(index).copied().unwrap_or(false)
    }

    /// Selects or clears one optimization. One already applied stays as it is.
    pub(crate) fn toggle(&mut self, index: usize) {
        if self.is_applied(index) {
            return;
        }
        if let Some(is_selected) = self.selected.get_mut(index) {
            *is_selected = !*is_selected;
            self.notice = None;
        }
    }

    /// How many optimizations are selected.
    pub(crate) fn selected_count(&self) -> usize {
        self.selected
            .iter()
            .filter(|is_selected| **is_selected)
            .count()
    }

    /// Whether something was applied that can be taken back.
    pub(crate) fn has_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Why the last request did nothing, when it did nothing.
    pub(crate) fn notice(&self) -> Option<&'static str> {
        self.notice
    }

    /// Opens the confirmation for the selected optimizations.
    pub(crate) fn request_apply(&mut self) {
        if self.selected_count() == 0 {
            self.notice = Some(pt_br::NETWORK_NOTHING_SELECTED);
        } else {
            self.notice = None;
            self.flow = ChangeFlow::Confirming(ChangeKind::Apply);
        }
    }

    /// Opens the confirmation for taking back what was applied.
    pub(crate) fn request_undo(&mut self) {
        if self.has_undo() {
            self.notice = None;
            self.flow = ChangeFlow::Confirming(ChangeKind::Undo);
        }
    }

    /// The changes a confirmation of `kind` would make, in order, with the
    /// positions of the optimizations they belong to.
    pub(crate) fn pending(&self, kind: ChangeKind) -> (Vec<NetworkChange>, Vec<usize>) {
        match kind {
            ChangeKind::Apply => self
                .optimizations
                .iter()
                .enumerate()
                .filter(|(index, _)| self.is_selected(*index) && !self.is_applied(*index))
                .map(|(index, offer)| (offer.change.clone(), index))
                .unzip(),
            ChangeKind::Undo => (self.undo.clone(), Vec::new()),
        }
    }

    /// Asks macOS to authenticate the user and makes the confirmed changes
    /// in the background.
    pub(crate) fn confirm(&mut self, ctx: &Context, services: &UiServices) {
        let ChangeFlow::Confirming(kind) = self.flow else {
            return;
        };
        let (changes, indexes) = self.pending(kind);
        let (tuner, requested) = (services.network_tuner.clone(), changes.clone());
        let task = Task::spawn(ctx, move |_, _| {
            TuneNetwork::new(tuner.as_ref()).execute(&requested, pt_br::NETWORK_ADMIN_PROMPT)
        });
        self.flow = ChangeFlow::Running {
            kind,
            changes,
            indexes,
            task,
        };
    }

    /// Closes the confirmation or the report of a change.
    pub(crate) fn close_flow(&mut self) {
        if !matches!(self.flow, ChangeFlow::Running { .. }) {
            self.flow = ChangeFlow::Closed;
        }
    }

    fn poll_change(&mut self) {
        let ChangeFlow::Running { task, .. } = &self.flow else {
            return;
        };
        let Some(outcome) = finished(task) else {
            return;
        };
        let ChangeFlow::Running {
            kind,
            changes,
            indexes,
            ..
        } = std::mem::replace(&mut self.flow, ChangeFlow::Closed)
        else {
            return;
        };
        self.flow = match outcome {
            Some(Ok(())) => {
                self.settle(kind, &indexes);
                ChangeFlow::Done(kind, changes)
            }
            Some(Err(PrivilegeError::Cancelled)) => {
                self.notice = Some(pt_br::NETWORK_CANCELLED);
                ChangeFlow::Closed
            }
            Some(Err(error)) => ChangeFlow::Failed(pt_br::network_privilege_error(&error)),
            None => ChangeFlow::Failed(pt_br::NETWORK_TASK_LOST.to_owned()),
        };
    }

    /// Records that the changes of `kind` were made.
    fn settle(&mut self, kind: ChangeKind, indexes: &[usize]) {
        match kind {
            ChangeKind::Apply => {
                for index in indexes {
                    if let Some(offer) = self.optimizations.get(*index) {
                        // The latest change is the first to be taken back.
                        if let Some(undo) = &offer.undo {
                            self.undo.insert(0, undo.clone());
                        }
                    }
                    if let Some(is_applied) = self.applied.get_mut(*index) {
                        *is_applied = true;
                    }
                    if let Some(is_selected) = self.selected.get_mut(*index) {
                        *is_selected = false;
                    }
                }
            }
            ChangeKind::Undo => {
                self.undo.clear();
                self.applied.fill(false);
            }
        }
    }
}

/// Looks at a task without waiting. `None` while it is still working;
/// `Some(None)` when its worker stopped without a result.
fn finished<T>(task: &Task<T>) -> Option<Option<T>> {
    match task.poll() {
        TaskPoll::Pending => None,
        TaskPoll::Ready(value) => Some(Some(value)),
        TaskPoll::Lost => Some(None),
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    use diskmarshal_application::ports::TrafficProbeError;
    use diskmarshal_domain::cleanup::RiskLevel;
    use diskmarshal_domain::network::{
        BitRate, NetworkChange, Optimization, OptimizationKind, ProgramTraffic, TrafficSnapshot,
    };
    use diskmarshal_domain::shared::ByteSize;

    use super::{ChangeFlow, ChangeKind, HISTORY_LENGTH, NetworkTab, NetworkViewModel};
    use crate::i18n::pt_br;

    fn program(name: &str, megabits: u64) -> ProgramTraffic {
        ProgramTraffic {
            name: name.to_owned(),
            download: BitRate::from_bits_per_second(megabits * 1_000_000),
            upload: BitRate::ZERO,
            received: ByteSize::ZERO,
            sent: ByteSize::ZERO,
            connections: Vec::new(),
        }
    }

    fn snapshot(megabits: u64, programs: Vec<ProgramTraffic>) -> TrafficSnapshot {
        TrafficSnapshot {
            download: BitRate::from_bits_per_second(megabits * 1_000_000),
            programs,
            ..TrafficSnapshot::default()
        }
    }

    fn offer(
        kind: OptimizationKind,
        change: NetworkChange,
        undo: Option<NetworkChange>,
    ) -> Optimization {
        Optimization {
            kind,
            change,
            undo,
            risk: RiskLevel::Safe,
            is_recommended: true,
        }
    }

    fn proxy(is_enabled: bool) -> NetworkChange {
        NetworkChange::SetProxyAutoDiscovery {
            service: "Wi-Fi".to_owned(),
            is_enabled,
        }
    }

    /// A model with two optimizations offered and selected.
    fn with_offers() -> NetworkViewModel {
        NetworkViewModel {
            optimizations: vec![
                offer(
                    OptimizationKind::DisableProxyDiscovery,
                    proxy(false),
                    Some(proxy(true)),
                ),
                offer(
                    OptimizationKind::FlushDnsCache,
                    NetworkChange::FlushDnsCache,
                    None,
                ),
            ],
            selected: vec![true, true],
            applied: vec![false, false],
            ..NetworkViewModel::default()
        }
    }

    #[test]
    fn keeps_only_the_most_recent_readings_and_scales_the_plot_to_the_peak() {
        let mut model = NetworkViewModel::default();
        model.record(Ok(snapshot(0, Vec::new())));
        for index in 0..(HISTORY_LENGTH as u64 + 20) {
            model.record(Ok(snapshot(index, Vec::new())));
        }

        let plot = model.plot();

        assert_eq!(plot.download.len(), HISTORY_LENGTH);
        assert!(
            (plot.top - 100.0).abs() < f32::EPSILON,
            "79 Mb/s with room above"
        );
        assert!((plot.download[HISTORY_LENGTH - 1] - 0.79).abs() < 1e-6);
        assert!((plot.download[0] - 0.20).abs() < 1e-6);
    }

    #[test]
    fn an_idle_connection_keeps_a_readable_scale() {
        let mut model = NetworkViewModel::default();
        model.record(Ok(snapshot(0, Vec::new())));
        assert!((model.plot().top - 10.0).abs() < f32::EPSILON);
    }

    #[test]
    fn a_failed_reading_keeps_the_last_good_one_on_screen() {
        let mut model = NetworkViewModel::default();
        model.record(Ok(snapshot(5, vec![program("Browser", 5)])));
        model.record(Err(TrafficProbeError::TimedOut));

        assert!(model.traffic().is_some());
        assert!(model.traffic_failure().is_some());
        model.record(Ok(snapshot(5, Vec::new())));
        assert!(model.traffic_failure().is_none());
    }

    #[test]
    fn shows_the_picked_program_while_it_has_traffic_and_the_busiest_otherwise() {
        let mut model = NetworkViewModel::default();
        assert!(model.shown_program().is_none());
        model.record(Ok(snapshot(
            9,
            vec![program("Browser", 8), program("Music", 1)],
        )));
        assert_eq!(model.shown_program().unwrap().name, "Browser");

        model.select_program("Music");
        assert_eq!(model.shown_program().unwrap().name, "Music");

        model.record(Ok(snapshot(8, vec![program("Browser", 8)])));
        assert_eq!(model.shown_program().unwrap().name, "Browser");
    }

    #[test]
    fn asking_to_apply_nothing_says_so_instead_of_opening_the_confirmation() {
        let mut model = with_offers();
        model.toggle(0);
        model.toggle(1);

        model.request_apply();

        assert!(matches!(model.flow, ChangeFlow::Closed));
        assert_eq!(model.notice(), Some(pt_br::NETWORK_NOTHING_SELECTED));

        model.toggle(1);
        assert_eq!(model.notice(), None);
        model.request_apply();
        assert!(matches!(
            model.flow,
            ChangeFlow::Confirming(ChangeKind::Apply)
        ));
        assert_eq!(
            model.pending(ChangeKind::Apply),
            (vec![NetworkChange::FlushDnsCache], vec![1])
        );
    }

    #[test]
    fn what_was_applied_can_be_taken_back_the_latest_change_first() {
        let mut model = with_offers();
        model.optimizations.push(offer(
            OptimizationKind::UseFasterDns {
                provider: "Cloudflare",
                current: Duration::from_millis(38),
                fastest: Duration::from_millis(9),
            },
            NetworkChange::SetDnsServers {
                service: "Wi-Fi".to_owned(),
                servers: vec![IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))],
            },
            Some(NetworkChange::SetDnsServers {
                service: "Wi-Fi".to_owned(),
                servers: Vec::new(),
            }),
        ));
        model.selected.push(true);
        model.applied.push(false);
        assert!(!model.has_undo());

        model.settle(ChangeKind::Apply, &[0, 1, 2]);

        assert!(model.is_applied(0) && model.is_applied(1) && model.is_applied(2));
        assert_eq!(model.selected_count(), 0);
        let (undo, _) = model.pending(ChangeKind::Undo);
        assert_eq!(undo.len(), 2, "flushing the cache has nothing to take back");
        assert!(matches!(undo[0], NetworkChange::SetDnsServers { .. }));
        assert_eq!(undo[1], proxy(true));

        model.toggle(0);
        assert!(
            !model.is_selected(0),
            "an applied change cannot be selected again"
        );

        model.settle(ChangeKind::Undo, &[]);
        assert!(!model.has_undo());
        assert!(!model.is_applied(0));
    }

    #[test]
    fn finds_a_tab_by_its_identifier() {
        assert_eq!(
            NetworkTab::from_slug("analysis"),
            Some(NetworkTab::Analysis)
        );
        assert_eq!(NetworkTab::from_slug("tuning"), Some(NetworkTab::Tuning));
        assert_eq!(NetworkTab::from_slug("nope"), None);
        assert_eq!(NetworkTab::ALL.len(), 3);
    }
}
