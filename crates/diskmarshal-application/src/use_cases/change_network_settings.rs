//! Use case: change network settings named by the caller, with no preview.
//! This is what the privileged helper runs.

use diskmarshal_domain::network::NetworkChange;

use crate::ApplicationError;
use crate::ports::NetworkSettings;

/// Most changes one request may carry.
const MAX_CHANGES: usize = 8;

/// Applies network changes after checking each one against this Mac.
pub struct ChangeNetworkSettings<'a> {
    settings: &'a dyn NetworkSettings,
}

impl<'a> ChangeNetworkSettings<'a> {
    /// Creates the use case over the settings of this Mac.
    #[must_use]
    pub fn new(settings: &'a dyn NetworkSettings) -> Self {
        Self { settings }
    }

    /// Applies `changes` in order.
    ///
    /// Every change is checked before the first one is applied: it must be
    /// well formed and, when it names a network service, that service must
    /// be one macOS lists. One change that fails a check stops the request
    /// with nothing touched.
    ///
    /// # Errors
    ///
    /// Fails when the request is empty, too long or names something that is
    /// not allowed, and when macOS refuses a change. Changes applied before
    /// a refused one stay applied.
    pub fn execute(&self, changes: &[NetworkChange]) -> Result<(), ApplicationError> {
        let refused = |reason: &str| ApplicationError::NetworkChangeNotAllowed(reason.to_owned());
        if changes.is_empty() {
            return Err(refused("no change requested"));
        }
        if changes.len() > MAX_CHANGES {
            return Err(refused("too many changes"));
        }
        if !changes.iter().all(NetworkChange::is_well_formed) {
            return Err(refused("malformed change"));
        }
        let services = self
            .settings
            .services()
            .map_err(|error| ApplicationError::NetworkChangeFailed(error.to_string()))?;
        for service in changes.iter().filter_map(NetworkChange::service) {
            if !services.iter().any(|known| known == service) {
                return Err(ApplicationError::NetworkChangeNotAllowed(format!(
                    "unknown network service: {service}"
                )));
            }
        }

        for change in changes {
            self.settings
                .apply(change)
                .map_err(|error| ApplicationError::NetworkChangeFailed(error.to_string()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Mutex;

    use diskmarshal_domain::network::NetworkChange;

    use super::ChangeNetworkSettings;
    use crate::ApplicationError;
    use crate::ports::{NetworkSettings, NetworkSettingsError};

    /// Settings that record what was applied and refuse one kind of change.
    #[derive(Default)]
    struct RecordingSettings {
        applied: Mutex<Vec<NetworkChange>>,
        refuses_dns: bool,
    }

    impl NetworkSettings for RecordingSettings {
        fn services(&self) -> Result<Vec<String>, NetworkSettingsError> {
            Ok(vec!["Wi-Fi".to_owned(), "Thunderbolt Bridge".to_owned()])
        }

        fn apply(&self, change: &NetworkChange) -> Result<(), NetworkSettingsError> {
            if self.refuses_dns && matches!(change, NetworkChange::SetDnsServers { .. }) {
                return Err(NetworkSettingsError("refused".to_owned()));
            }
            self.applied.lock().unwrap().push(change.clone());
            Ok(())
        }
    }

    fn proxy(service: &str) -> NetworkChange {
        NetworkChange::SetProxyAutoDiscovery {
            service: service.to_owned(),
            is_enabled: false,
        }
    }

    fn dns(service: &str) -> NetworkChange {
        NetworkChange::SetDnsServers {
            service: service.to_owned(),
            servers: vec![IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))],
        }
    }

    #[test]
    fn applies_the_changes_in_the_order_they_were_asked() {
        let settings = RecordingSettings::default();
        let changes = [proxy("Wi-Fi"), NetworkChange::FlushDnsCache, dns("Wi-Fi")];

        ChangeNetworkSettings::new(&settings)
            .execute(&changes)
            .unwrap();

        assert_eq!(*settings.applied.lock().unwrap(), changes);
    }

    #[test]
    fn one_change_that_is_not_allowed_stops_the_request_with_nothing_touched() {
        let settings = RecordingSettings::default();
        let run =
            |changes: &[NetworkChange]| ChangeNetworkSettings::new(&settings).execute(changes);
        let too_many = vec![NetworkChange::FlushDnsCache; 9];

        for changes in [
            &[][..],
            &too_many[..],
            &[NetworkChange::FlushDnsCache, proxy("Ethernet 9")][..],
            &[NetworkChange::FlushDnsCache, proxy("-setdnsservers")][..],
        ] {
            assert!(matches!(
                run(changes),
                Err(ApplicationError::NetworkChangeNotAllowed(_))
            ));
        }
        assert!(settings.applied.lock().unwrap().is_empty());
    }

    #[test]
    fn a_change_macos_refuses_stops_the_ones_after_it() {
        let settings = RecordingSettings {
            refuses_dns: true,
            ..RecordingSettings::default()
        };
        let changes = [NetworkChange::FlushDnsCache, dns("Wi-Fi"), proxy("Wi-Fi")];

        let result = ChangeNetworkSettings::new(&settings).execute(&changes);

        assert!(matches!(
            result,
            Err(ApplicationError::NetworkChangeFailed(_))
        ));
        assert_eq!(
            *settings.applied.lock().unwrap(),
            [NetworkChange::FlushDnsCache]
        );
    }
}
