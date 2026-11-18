//! Use case: change network settings the user approved, after macOS
//! authenticates them. This is what the app runs.

use diskmarshal_domain::network::NetworkChange;

use crate::ports::{NetworkTuner, PrivilegeError};

/// Asks for approved network changes to be applied with administrator rights.
pub struct TuneNetwork<'a> {
    tuner: &'a dyn NetworkTuner,
}

impl<'a> TuneNetwork<'a> {
    /// Creates the use case over the tuner.
    #[must_use]
    pub fn new(tuner: &'a dyn NetworkTuner) -> Self {
        Self { tuner }
    }

    /// Asks macOS to authenticate the user, showing `prompt` in its dialog,
    /// and applies `changes` in order. Taking changes back is the same
    /// call, with the changes that undo them.
    ///
    /// # Errors
    ///
    /// Fails, before the user is asked to authenticate for nothing, when
    /// there is no change or one is malformed. Fails as well when the user
    /// does not authenticate or a change is refused.
    pub fn execute(&self, changes: &[NetworkChange], prompt: &str) -> Result<(), PrivilegeError> {
        if changes.is_empty() || !changes.iter().all(NetworkChange::is_well_formed) {
            return Err(PrivilegeError::Failed("invalid network change".to_owned()));
        }
        self.tuner.apply(changes, prompt)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use diskmarshal_domain::network::NetworkChange;

    use super::TuneNetwork;
    use crate::ports::{NetworkTuner, PrivilegeError};

    #[derive(Default)]
    struct RecordingTuner(Mutex<Vec<Vec<NetworkChange>>>);

    impl NetworkTuner for RecordingTuner {
        fn is_available(&self) -> bool {
            true
        }

        fn apply(&self, changes: &[NetworkChange], _prompt: &str) -> Result<(), PrivilegeError> {
            self.0.lock().unwrap().push(changes.to_vec());
            Ok(())
        }
    }

    #[test]
    fn hands_the_approved_changes_to_the_tuner_in_one_request() {
        let tuner = RecordingTuner::default();
        let changes = [
            NetworkChange::FlushDnsCache,
            NetworkChange::SetProxyAutoDiscovery {
                service: "Wi-Fi".to_owned(),
                is_enabled: false,
            },
        ];

        TuneNetwork::new(&tuner)
            .execute(&changes, "prompt")
            .unwrap();

        assert_eq!(*tuner.0.lock().unwrap(), [changes.to_vec()]);
    }

    #[test]
    fn refuses_a_bad_request_before_anyone_is_asked_to_authenticate() {
        let tuner = RecordingTuner::default();
        let malformed = NetworkChange::SetProxyAutoDiscovery {
            service: "--force".to_owned(),
            is_enabled: false,
        };

        for changes in [&[][..], &[NetworkChange::FlushDnsCache, malformed][..]] {
            assert!(matches!(
                TuneNetwork::new(&tuner).execute(changes, "prompt"),
                Err(PrivilegeError::Failed(_))
            ));
        }
        assert!(tuner.0.lock().unwrap().is_empty());
    }
}
