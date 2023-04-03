use openbrush::contracts::psp22::PSP22Error;
use openbrush::traits::Balance;

#[openbrush::wrapper]
pub type Psp22FeeRef = dyn Psp22Fee;

#[openbrush::trait_definition]
pub trait Psp22Fee {
    #[ink(message)]
    fn set_max_wallet(&mut self, max_wallet: u128) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn set_max_tx(&mut self, max_tx: u128) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn get_max_wallet(&mut self) -> Balance;

    #[ink(message)]
    fn get_max_tx(&mut self) -> Balance;

    #[ink(message)]
    fn set_fee(&mut self, fee: u8) -> Result<(), PSP22Error>;

    #[ink(message)]
    fn get_fee(&mut self) -> u8;
}
