//! Network tuner that asks macOS to authenticate the user and to run the
//! helper once.

use std::ffi::OsString;
use std::iter;
use std::path::PathBuf;

use diskmarshal_application::ports::{AuthenticationKind, NetworkTuner, PrivilegeError};
use diskmarshal_domain::network::NetworkChange;

use crate::privilege::helper_launcher::HelperLauncher;
use crate::privilege::network_request::{NETWORK_FLAG, encode_network_changes};

/// Changes network settings through the privileged helper, once per request.
///
/// The helper receives what to change, never how: it turns each change into
/// the fixed command for it and checks every network service against the
/// ones macOS lists.
#[derive(Debug, Clone)]
pub struct MacosNetworkTuner {
    launcher: HelperLauncher,
}

impl MacosNetworkTuner {
    /// Looks for the helper beside the running executable. Without it the
    /// tuner reports itself unavailable.
    #[must_use]
    pub fn beside_current_executable() -> Self {
        Self {
            launcher: HelperLauncher::beside_current_executable(),
        }
    }

    /// Uses the helper at `helper`, when it is there and can be trusted.
    #[must_use]
    pub fn new(helper: Option<PathBuf>) -> Self {
        Self {
            launcher: HelperLauncher::new(helper),
        }
    }
}

impl NetworkTuner for MacosNetworkTuner {
    fn is_available(&self) -> bool {
        self.launcher.is_available()
    }

    fn authentication(&self) -> AuthenticationKind {
        self.launcher.authentication()
    }

    fn apply(&self, changes: &[NetworkChange], prompt: &str) -> Result<(), PrivilegeError> {
        // Checked before the user is asked to authenticate for nothing.
        if changes.is_empty() || !changes.iter().all(NetworkChange::is_well_formed) {
            return Err(PrivilegeError::Failed("invalid network change".to_owned()));
        }
        let arguments: Vec<OsString> = iter::once(NETWORK_FLAG.to_owned())
            .chain(encode_network_changes(changes))
            .map(OsString::from)
            .collect();
        self.launcher.run(&arguments, prompt).map(drop)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use diskmarshal_application::ports::{NetworkTuner, PrivilegeError};
    use diskmarshal_domain::network::NetworkChange;
    use diskmarshal_test_support::TempTree;

    use super::MacosNetworkTuner;

    #[test]
    fn is_unavailable_without_a_helper() {
        let fixture = TempTree::new().unwrap();
        let tuner = MacosNetworkTuner::new(Some(fixture.path("missing")));

        assert!(!tuner.is_available());
        assert!(matches!(
            tuner.apply(&[NetworkChange::FlushDnsCache], "prompt"),
            Err(PrivilegeError::Unavailable(_))
        ));
        assert!(!MacosNetworkTuner::new(None).is_available());
    }

    #[test]
    fn refuses_a_bad_request_before_asking_for_authentication() {
        let fixture = TempTree::new().unwrap();
        let helper = fixture.file("helper", 16).unwrap();
        fs::set_permissions(&helper, fs::Permissions::from_mode(0o755)).unwrap();
        let tuner = MacosNetworkTuner::new(Some(helper));
        assert!(tuner.is_available());
        let as_an_option = NetworkChange::SetProxyAutoDiscovery {
            service: "--help".to_owned(),
            is_enabled: false,
        };

        for changes in [&[][..], &[NetworkChange::FlushDnsCache, as_an_option][..]] {
            assert!(matches!(
                tuner.apply(changes, "prompt"),
                Err(PrivilegeError::Failed(_))
            ));
        }
    }
}
