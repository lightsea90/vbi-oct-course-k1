use crate::{mock::*, Error};
use super::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn create_cert_works() {
    new_test_ext().execute_with(||{
        let signature = vec![0,1,2];
        let expired_timestamp = 321u128; 
        let cert = Cert {
            signature: signature.clone(),
            expired_timestamp: expired_timestamp,
        };

        assert_ok!(TemplateModule::create_cert(Origin::signed(1), signature, expired_timestamp));

        assert_eq!(
            Proofs::<Test>::get(&cert),
            (1, frame_system::Pallet::<Test>::block_number()),
        );
    })
}

#[test]
fn create_cert_failed_when_cert_already_exists() {
    new_test_ext().execute_with(||{
        let signature = vec![0,1,2];
        let expired_timestamp = 321u128; 
        // let cert = Cert {
        //     signature: signature,
        //     expired_timestamp: expired_timestamp,
        // };
        let _ = TemplateModule::create_cert(Origin::signed(1), signature.clone(), expired_timestamp);
        assert_noop!(
            TemplateModule::create_cert(Origin::signed(1), signature, expired_timestamp),
            Error::<Test>::ProofAlreadyClaimed,
        );
    })
}

