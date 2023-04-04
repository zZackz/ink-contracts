#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod my_psp22 {
    // imports from openbrush
    use ink::codegen::Env;
    use ink::prelude::vec::Vec;
    use logics_pkg::traits::psp22_fee::*;
    use logics_pkg::{impls::psp22_fee::*, traits::psp22_fee::*};
    use openbrush::contracts::ownable::*;
    use openbrush::contracts::psp22::extensions::metadata::*;
    use openbrush::traits::Storage;
    use openbrush::traits::String;
    use openbrush::traits::ZERO_ADDRESS;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Contract {
        #[storage_field]
        psp22: psp22::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        psp22_fee: psp22_fee::Data,
    }

    // Section contains default implementation without any modifications

    impl Ownable for Contract {}
    impl PSP22Metadata for Contract {}

    impl Contract {
        #[ink(constructor)]
        pub fn new(
            initial_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimal: u8,
            max_wallet: u128,
            max_tx: u128,
            fee: u128,
        ) -> Self {
            let mut _instance = Self::default();
            _instance
                ._mint_to(_instance.env().caller(), initial_supply)
                .expect("Should mint");
            _instance._init_with_owner(_instance.env().caller());
            _instance.metadata.name = name;
            _instance.metadata.symbol = symbol;
            _instance.metadata.decimals = decimal;
            _instance.psp22_fee.max_wallet = initial_supply * max_wallet / 100;
            _instance.psp22_fee.max_tx = initial_supply * max_tx / 100;
            _instance.psp22_fee.fee = fee;
            _instance
        }
    }

    impl Psp22Fee for Contract {}
    impl PSP22 for Contract {
        #[ink(message)]
        fn transfer(&mut self, to: AccountId, value: Balance, data: Vec<u8>) -> Result<(), PSP22Error> {
            let from = self.env().caller();

            let balance_wallet = self._balance_of(&to);

            if value > self.psp22_fee.max_tx {
                return Err(PSP22Error::InsufficientBalance);
            }

            if value + balance_wallet > self.psp22_fee.max_wallet {
                return Err(PSP22Error::InsufficientBalance);
            }

            let is_tax = to != ZERO_ADDRESS.into() && from != ZERO_ADDRESS.into();

            let tax = if is_tax { (value * self.psp22_fee.fee) / 100 } else { 0 };

            self._transfer_from_to(from, self.ownable.owner, tax, data.clone())?;
            self._transfer_from_to(from, to, value - tax, data)?;
            Ok(())
        }
    }
}
