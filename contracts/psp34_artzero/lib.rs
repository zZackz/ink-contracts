#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]
pub use self::psp34_nft::{Psp34Nft, Psp34NftRef};

#[allow(clippy::let_unit_value)]
#[allow(clippy::inline_fn_without_body)]
#[allow(clippy::too_many_arguments)]
#[openbrush::contract]
pub mod psp34_nft {

    use ink::codegen::{EmitEvent, Env};
    use ink::prelude::{string::String, vec::Vec};
    use logics_pkg::impls::psp34_artzero_support::admin::*;
    use logics_pkg::impls::psp34_artzero_support::psp34_traits::psp34_traits::InternalTraits;
    use logics_pkg::impls::psp34_artzero_support::psp34_traits::*;
    use logics_pkg::traits::{admin::*, error::Error, psp34_traits::*};
    use openbrush::{
        contracts::ownable::*,
        contracts::psp34::extensions::{burnable::*, enumerable::*, metadata::*},
        modifiers,
        traits::{DefaultEnv, Storage},
    };

    #[derive(Default, Storage)]
    #[ink(storage)]
    pub struct Psp34Nft {
        #[storage_field]
        psp34: psp34::Data<enumerable::Balances>,
        #[storage_field]
        metadata: metadata::Data,
        #[storage_field]
        ownable: ownable::Data,
        #[storage_field]
        manager: psp34_traits::Manager,
        #[storage_field]
        admin_data: admin::data::Data,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: Id,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: Option<Id>,
        approved: bool,
    }

    impl Ownable for Psp34Nft {}
    impl PSP34 for Psp34Nft {}
    impl PSP34Metadata for Psp34Nft {}
    impl PSP34Enumerable for Psp34Nft {}
    impl Psp34Traits for Psp34Nft {}
    impl AdminTrait for Psp34Nft {}
    impl InternalTraits for Psp34Nft {}

    impl PSP34Burnable for Psp34Nft {
        #[ink(message)]
        fn burn(&mut self, account: AccountId, id: Id) -> Result<(), PSP34Error> {
            let caller = Self::env().caller();
            let token_owner = self.owner_of(id.clone()).unwrap();
            if token_owner != account {
                return Err(PSP34Error::Custom(String::from("not token owner").into_bytes()));
            }

            let allowance = self.allowance(account, caller, Some(id.clone()));

            if caller == account || allowance {
                self._burn_from(account, id)
            } else {
                Err(PSP34Error::Custom(
                    String::from("caller is not token owner or approved").into_bytes(),
                ))
            }
        }
    }

    impl Psp34Nft {
        #[ink(constructor)]
        pub fn new(name: String, symbol: String, base_uri: String, max_supply: u64, price_per_mint: Balance) -> Self {
            let mut instance = Self::default();

            instance._init_with_owner(instance.env().caller());
            instance._set_attribute(Id::U8(0), String::from("name").into_bytes(), name.into_bytes());
            instance._set_attribute(Id::U8(0), String::from("symbol").into_bytes(), symbol.into_bytes());
            instance._set_attribute(Id::U8(0), String::from("baseURI").into_bytes(), base_uri.into_bytes());
            instance.manager.max_supply = max_supply;
            instance.manager.price_per_mint = price_per_mint;
            instance.manager.last_token_id = 0;
            instance.manager.max_amount = 1;
            instance
        }

        /// This function let NFT Contract Owner to mint a new NFT without providing NFT Traits/Attributes
        #[ink(message, payable)]
        pub fn mint(&mut self, mint_amount: u64) -> Result<(), Error> {
            self.check_amount(mint_amount)?;
            self.check_value(self.env().transferred_value(), mint_amount)?;
            let next_to_mint = self.manager.last_token_id + 1; // first mint id is 1
            let mint_offset = next_to_mint + mint_amount;
            let caller = self.env().caller();
            // self.manager.last_token_id = self.manager.last_token_id.checked_add(1).unwrap();

            for mint_id in next_to_mint..mint_offset {
                if self._mint_to(caller, Id::U64(mint_id)).is_err() {
                    return Err(Error::Custom(String::from("Cannot mint")));
                }
                self.manager.last_token_id += 1;
                self._emit_transfer_event(None, Some(caller), Id::U64(mint_id));
            }
            // if self._mint_to(caller, Id::U64(self.manager.last_token_id)).is_err() {
            //     return Err(Error::Custom(String::from("Cannot mint")));
            // }
            Ok(())
        }
    }

    // Override event emission methods
    impl psp34::Internal for Psp34Nft {
        fn _emit_transfer_event(&self, from: Option<AccountId>, to: Option<AccountId>, id: Id) {
            self.env().emit_event(Transfer { from, to, id });
        }

        fn _emit_approval_event(&self, from: AccountId, to: AccountId, id: Option<Id>, approved: bool) {
            self.env().emit_event(Approval { from, to, id, approved });
        }
    }
}
