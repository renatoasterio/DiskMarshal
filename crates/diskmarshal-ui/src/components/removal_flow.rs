//! The steps every removal goes through: confirm, run, report.
//!
//! Every screen that deletes uses this one flow, and the flow always removes
//! through the `DeleteItems` use case.

use std::path::PathBuf;

use diskmarshal_application::use_cases::{
    CleanSystemAreas, DeleteItems, DeletionPorts, DeletionReport, ExecutionMode,
};
use diskmarshal_domain::deletion::{DeletionRequest, DeletionStatus, ProtectedPaths};
use egui::{Context, RichText, Ui};

use crate::app::UiServices;
use crate::components::{ButtonKind, activity_bar, dialog, hud_button};
use crate::i18n::{Counted, pt_br};
use crate::screens::ScreenContext;
use crate::tasks::{Task, TaskPoll};
use crate::theme::{FONT_SIZE_DATA, FONT_SIZE_SMALL, SPACE_2, SPACE_3, Tokens};

/// Where a removal is.
pub(crate) enum RemovalFlow {
    Closed,
    /// Waiting for the user to authorize the removal.
    Confirm,
    /// The removal is running.
    Running(Task<RemovalResult>),
    /// The removal ended; this is what happened.
    Done(RemovalResult),
}

/// What a removal did.
pub(crate) struct RemovalResult {
    report: DeletionReport,
    /// What the user must know besides the totals, such as why the items
    /// that need an administrator were left.
    warnings: Vec<String>,
    /// What is said about the items that were left as they were.
    skipped_explanation: Counted,
}

impl Default for RemovalResult {
    fn default() -> Self {
        Self::new(DeletionReport::default(), Vec::new())
    }
}

impl RemovalResult {
    /// The result of a removal, with what the user must know besides it.
    pub(crate) fn new(report: DeletionReport, warnings: Vec<String>) -> Self {
        Self {
            report,
            warnings,
            skipped_explanation: pt_br::ITEMS_SKIPPED,
        }
    }

    /// Changes what is said about the items that were left as they were,
    /// for a removal where macOS is not who refuses.
    pub(crate) fn explaining_skipped(mut self, explanation: Counted) -> Self {
        self.skipped_explanation = explanation;
        self
    }
}

/// What the confirmation dialog says.
pub(crate) struct ConfirmText<'a> {
    pub body: &'a str,
    /// Short facts under the body: how much, where it goes.
    pub lines: &'a [String],
    /// Label of the button that authorizes the removal.
    pub action: &'a str,
}

/// What the user did in the flow during this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemovalEvent {
    Idle,
    /// The user authorized the removal. The screen must now call `start`.
    Confirmed,
    /// The user closed the report of a finished removal.
    Finished,
}

impl RemovalFlow {
    /// Opens the confirmation dialog.
    pub(crate) fn open(&mut self) {
        *self = Self::Confirm;
    }

    /// Removes `requests` in the background. Every item goes through the
    /// deletion policy again right before it is touched, and nothing outside
    /// `roots` can be removed.
    pub(crate) fn start(
        &mut self,
        ctx: &Context,
        services: &UiServices,
        requests: Vec<DeletionRequest>,
        roots: Vec<PathBuf>,
    ) {
        self.start_with_system(ctx, services, requests, roots, Vec::new());
    }

    /// Does what [`start`](Self::start) does and then cleans the system
    /// areas of the administrator rules named in `system_rules`. For those,
    /// macOS asks the user for Touch ID or the password in its own dialog.
    pub(crate) fn start_with_system(
        &mut self,
        ctx: &Context,
        services: &UiServices,
        requests: Vec<DeletionRequest>,
        roots: Vec<PathBuf>,
        system_rules: Vec<String>,
    ) {
        let services = services.clone();
        self.start_work(ctx, move || {
            let mut report = delete_items(&services, &requests, &roots);
            let mut warnings = Vec::new();
            if !system_rules.is_empty() {
                let cleaner =
                    CleanSystemAreas::new(services.privileged.as_ref(), services.history.as_ref());
                match cleaner.execute(&system_rules, pt_br::ADMIN_PROMPT) {
                    Ok(cleaned) => report.absorb(cleaned),
                    Err(error) => warnings.push(pt_br::privilege_error(&error)),
                }
            }
            RemovalResult::new(report, warnings)
        });
    }

    /// Runs `work` in the background as the removal of this flow. A screen
    /// whose removal is not made of files alone uses this.
    pub(crate) fn start_work(
        &mut self,
        ctx: &Context,
        work: impl FnOnce() -> RemovalResult + Send + 'static,
    ) {
        *self = Self::Running(Task::spawn(ctx, move |_, _| work()));
    }

    /// Draws the dialog of the current step, if any.
    pub(crate) fn show(
        &mut self,
        ctx: &Context,
        context: &ScreenContext<'_>,
        text: &ConfirmText<'_>,
    ) -> RemovalEvent {
        let tokens = context.tokens;
        if let Self::Running(removal) = self {
            match removal.poll() {
                TaskPoll::Pending => {}
                TaskPoll::Ready(result) => *self = Self::Done(result),
                // Only a crash of the worker gets here. An empty report says
                // truthfully that nothing is known to have been removed.
                TaskPoll::Lost => *self = Self::Done(RemovalResult::default()),
            }
        }
        match self {
            Self::Closed => RemovalEvent::Idle,
            Self::Confirm => {
                let (confirmed, cancelled) = dialog(ctx, tokens, pt_br::CONFIRM_TITLE, |ui| {
                    ui.label(RichText::new(text.body));
                    ui.add_space(SPACE_2);
                    for line in text.lines {
                        bullet(ui, tokens, line);
                    }
                    ui.add_space(SPACE_3);
                    let mut answer = (false, false);
                    ui.horizontal(|ui| {
                        answer.1 =
                            hud_button(ui, tokens, pt_br::CANCEL, ButtonKind::Quiet).clicked();
                        ui.add_space(SPACE_2);
                        answer.0 =
                            hud_button(ui, tokens, text.action, ButtonKind::Primary).clicked();
                    });
                    answer
                });
                if confirmed {
                    return RemovalEvent::Confirmed;
                }
                if cancelled {
                    *self = Self::Closed;
                }
                RemovalEvent::Idle
            }
            Self::Running(_) => {
                dialog(ctx, tokens, pt_br::REMOVING_TITLE, |ui| {
                    ui.label(
                        RichText::new(pt_br::REMOVING_BODY).color(tokens.color_text_secondary),
                    );
                    ui.add_space(SPACE_2);
                    let is_moving = context.appearance.motion.allows_ambient();
                    activity_bar(ui, tokens, context.now, is_moving);
                });
                RemovalEvent::Idle
            }
            Self::Done(result) => {
                let closed = dialog(ctx, tokens, pt_br::DONE_TITLE, |ui| {
                    summary(ui, tokens, result);
                    for warning in &result.warnings {
                        ui.add_space(SPACE_2);
                        ui.label(
                            RichText::new(warning)
                                .size(FONT_SIZE_SMALL)
                                .color(tokens.color_warn),
                        );
                    }
                    ui.add_space(SPACE_3);
                    hud_button(ui, tokens, pt_br::CLOSE, ButtonKind::Primary).clicked()
                });
                if closed {
                    *self = Self::Closed;
                    return RemovalEvent::Finished;
                }
                RemovalEvent::Idle
            }
        }
    }
}

/// Removes `requests` through the one use case that deletes files. Every
/// item goes through the deletion policy again right before it is touched,
/// and nothing outside `roots` can be removed.
pub(crate) fn delete_items(
    services: &UiServices,
    requests: &[DeletionRequest],
    roots: &[PathBuf],
) -> DeletionReport {
    let policy = ProtectedPaths::new(&services.home);
    let ports = DeletionPorts {
        inspector: services.inspector.as_ref(),
        remover: services.remover.as_ref(),
        trash_bin: services.trash_bin.as_ref(),
        history: services.history.as_ref(),
    };
    DeleteItems::new(&policy, ports).execute(requests, roots, ExecutionMode::Real)
}

/// Says plainly what a removal did, including what it left untouched.
fn summary(ui: &mut Ui, tokens: &Tokens, result: &RemovalResult) {
    let report = &result.report;
    let removed = report
        .outcomes
        .iter()
        .filter(|outcome| outcome.is_removed())
        .count();
    let skipped = report
        .outcomes
        .iter()
        .filter(|outcome| matches!(outcome.status, DeletionStatus::Skipped(_)))
        .count();
    // When everything went to the Trash nothing is free yet; saying "0 B freed"
    // would be true and useless.
    let headline = if report.reclaimed.is_zero() && !report.trashed.is_zero() {
        format!("{} {}", report.trashed, pt_br::IN_TRASH)
    } else {
        format!("{} {}", report.reclaimed, pt_br::FREED)
    };
    ui.label(
        RichText::new(headline)
            .size(FONT_SIZE_DATA + 13.0)
            .color(tokens.color_ok),
    );
    ui.add_space(SPACE_2);
    bullet(ui, tokens, &pt_br::ITEMS_REMOVED.of(removed));
    if !report.trashed.is_zero() {
        bullet(
            ui,
            tokens,
            &format!("{} {}", report.trashed, pt_br::SENT_TO_TRASH),
        );
        bullet(ui, tokens, pt_br::TRASH_FREES_LATER);
    }
    for (amount, explanation) in [
        (skipped, result.skipped_explanation),
        (report.unrecorded, pt_br::ITEMS_UNRECORDED),
    ] {
        if amount > 0 {
            ui.label(
                RichText::new(format!("· {}", explanation.of(amount)))
                    .size(FONT_SIZE_SMALL)
                    .color(tokens.color_warn),
            );
        }
    }
}

fn bullet(ui: &mut Ui, tokens: &Tokens, text: &str) {
    ui.label(
        RichText::new(format!("· {text}"))
            .size(FONT_SIZE_SMALL)
            .color(tokens.color_text_secondary),
    );
}
