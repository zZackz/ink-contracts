use crate::traits::psp22_fee::Psp22Fee;
use openbrush::contracts::ownable::*;
use openbrush::contracts::psp22::{self, PSP22Error};
use openbrush::traits::{Balance, Storage};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Data);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Data {
    pub max_wallet: Balance,
    pub max_tx: Balance,
    pub fee: u8,
}

impl<T> Psp22Fee for T
where
    T: Storage<Data> + Storage<psp22::Data> + Storage<ownable::Data>,
{
    #[openbrush::modifiers(only_owner)]
    fn set_max_wallet(&mut self, max_wallet: u128) -> Result<(), PSP22Error> {
        self.data::<Data>().max_wallet = (self.data::<psp22::Data>().supply * max_wallet) / 100;

        Ok(())
    }

    #[openbrush::modifiers(only_owner)]
    fn set_max_tx(&mut self, max_tx: u128) -> Result<(), PSP22Error> {
        self.data::<Data>().max_tx = (self.data::<psp22::Data>().supply * max_tx) / 100;

        Ok(())
    }

    fn get_max_wallet(&mut self) -> Balance {
        self.data::<Data>().max_wallet
    }

    fn get_max_tx(&mut self) -> Balance {
        self.data::<Data>().max_tx
    }

    #[openbrush::modifiers(only_owner)]
    fn set_fee(&mut self, fee: u8) -> Result<(), PSP22Error> {
        self.data::<Data>().fee = fee;

        Ok(())
    }

    fn get_fee(&mut self) -> u8 {
        self.data::<Data>().fee
    }
}
