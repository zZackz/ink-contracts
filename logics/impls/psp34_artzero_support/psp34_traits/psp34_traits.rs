use crate::traits::error::Error;
pub use crate::traits::psp34_traits::*;
use ink::prelude::{
    string::{String, ToString},
    vec::Vec,
};
use openbrush::{
    contracts::ownable::*,
    contracts::psp34::extensions::{enumerable::*, metadata::*},
    modifier_definition, modifiers,
    storage::Mapping,
    traits::{AccountId, Balance, Storage},
};

pub const STORAGE_KEY: u32 = openbrush::storage_unique_key!(Manager);

#[derive(Default, Debug)]
#[openbrush::upgradeable_storage(STORAGE_KEY)]
pub struct Manager {
    pub last_token_id: u64,
    pub attribute_count: u32,
    pub attribute_names: Mapping<u32, Vec<u8>>,
    pub locked_tokens: Mapping<Id, bool>,
    pub locked_token_count: u64,
    pub price_per_mint: Balance,
    pub max_amount: u64,
    pub max_supply: u64,
    pub _reserved: Option<()>,
}

pub trait InternalTraits {
    /// Check if the transferred mint values is as expected
    fn check_value(&self, transferred_value: u128, mint_amount: u64) -> Result<(), Error>;

    /// Check amount of tokens to be minted
    fn check_amount(&self, mint_amount: u64) -> Result<(), Error>;

    /// Check if token is minted
    fn token_exists(&self, id: Id) -> Result<(), Error>;
}

#[modifier_definition]
pub fn only_token_owner<T, F, R, E>(
    instance: &mut T,
    body: F,
    token_owner: AccountId,
) -> Result<R, E>
where
    T: Storage<Manager>,
    F: FnOnce(&mut T) -> Result<R, E>,
    E: From<Error>,
{
    if token_owner != T::env().caller() {
        return Err(From::from(Error::NotTokenOwner));
    }
    body(instance)
}

impl<T: Storage<Manager>> Psp34Traits for T
where
    T: PSP34
        + psp34::Internal
        + Storage<psp34::extensions::metadata::Data>
        + Storage<psp34::Data<psp34::extensions::enumerable::Balances>>
        + Storage<ownable::Data>,
{
    /// Get Token Count
    default fn get_last_token_id(&self) -> u64 {
        return self.data::<Manager>().last_token_id;
    }

    /// Lock nft - Only owner token
    #[modifiers(only_token_owner(self.owner_of(token_id.clone()).unwrap()))]
    default fn lock(&mut self, token_id: Id) -> Result<(), Error> {
        self.data::<Manager>().locked_token_count = self
            .data::<Manager>()
            .locked_token_count
            .checked_add(1)
            .unwrap();
        self.data::<Manager>()
            .locked_tokens
            .insert(&token_id, &true);
        Ok(())
    }

    /// Check token is locked or not
    default fn is_locked_nft(&self, token_id: Id) -> bool {
        if self
            .data::<Manager>()
            .locked_tokens
            .get(&token_id)
            .is_some()
        {
            return true;
        }
        return false;
    }

    /// Get Locked Token Count
    default fn get_locked_token_count(&self) -> u64 {
        self.data::<Manager>().locked_token_count
    }

    /// Change baseURI
    #[modifiers(only_owner)]
    default fn set_base_uri(&mut self, uri: String) -> Result<(), Error> {
        self._set_attribute(
            Id::U8(0),
            String::from("baseURI").into_bytes(),
            uri.into_bytes(),
        );
        Ok(())
    }

    /// Only Owner can set multiple attributes to a token
    #[modifiers(only_owner)]
    default fn set_multiple_attributes(
        &mut self,
        token_id: Id,
        metadata: Vec<(String, String)>,
    ) -> Result<(), Error> {
        if token_id == Id::U64(0) {
            return Err(Error::InvalidInput);
        }
        if self.is_locked_nft(token_id.clone()) {
            return Err(Error::Custom(String::from("Token is locked")));
        }
        for (attribute, value) in &metadata {
            add_attribute_name(self, &attribute.clone().into_bytes());
            self._set_attribute(
                token_id.clone(),
                attribute.clone().into_bytes(),
                value.clone().into_bytes(),
            );
        }
        Ok(())
    }

    /// Get multiple  attributes
    default fn get_attributes(&self, token_id: Id, attributes: Vec<String>) -> Vec<String> {
        let length = attributes.len();
        let mut ret = Vec::<String>::new();
        for i in 0..length {
            let attribute = attributes[i].clone();
            let value = self.get_attribute(token_id.clone(), attribute.into_bytes());
            if value.is_some() {
                ret.push(String::from_utf8(value.unwrap()).unwrap());
            } else {
                ret.push(String::from(""));
            }
        }
        ret
    }

    /// Get Attribute Count
    default fn get_attribute_count(&self) -> u32 {
        self.data::<Manager>().attribute_count
    }
    /// Get Attribute Name
    default fn get_attribute_name(&self, index: u32) -> String {
        let attribute = self.data::<Manager>().attribute_names.get(&index);
        if attribute.is_some() {
            String::from_utf8(attribute.unwrap()).unwrap()
        } else {
            String::from("")
        }
    }

    /// Get URI from token ID
    default fn token_uri(&self, token_id: u64) -> String {
        let value = self.get_attribute(Id::U8(0), String::from("baseURI").into_bytes());
        let mut token_uri = String::from_utf8(value.unwrap()).unwrap();
        token_uri = token_uri + &token_id.to_string() + &String::from(".json");
        token_uri
    }

    /// Get owner address
    default fn get_owner(&self) -> AccountId {
        self.owner()
    }

    /// Withdraws funds to contract owner
    #[modifiers(only_owner)]
    default fn withdraw(&mut self) -> Result<(), Error> {
        let balance = Self::env().balance();
        let current_balance = balance
            .checked_sub(Self::env().minimum_balance())
            .unwrap_or_default();
        Self::env()
            .transfer(self.data::<ownable::Data>().owner(), current_balance)
            .map_err(|_| Error::Custom(String::from("Withdrawal Failed")))?;

        Ok(())
    }

    /// Set max number of tokens which could be minted per call
    #[modifiers(only_owner)]
    default fn set_max_mint_amount(&mut self, max_amount: u64) -> Result<(), Error> {
        self.data::<Manager>().max_amount = max_amount;
        Ok(())
    }

    /// Get max supply of tokens
    default fn max_supply(&self) -> u64 {
        self.data::<Manager>().max_supply
    }

    /// Get token price
    default fn price(&self) -> Balance {
        self.data::<Manager>().price_per_mint
    }

    /// Get max number of tokens which could be minted per call
    default fn get_max_mint_amount(&mut self) -> u64 {
        self.data::<Manager>().max_amount
    }
}

fn add_attribute_name<T: Storage<Manager>>(instance: &mut T, attribute_input: &Vec<u8>) {
    let mut exist: bool = false;
    for index in 0..instance.data::<Manager>().attribute_count {
        let attribute_name = instance.data::<Manager>().attribute_names.get(&(index + 1));
        if attribute_name.is_some() {
            if attribute_name.unwrap() == *attribute_input {
                exist = true;
                break;
            }
        }
    }
    if !exist {
        instance.data::<Manager>().attribute_count = instance
            .data::<Manager>()
            .attribute_count
            .checked_add(1)
            .unwrap();
        let data = &mut instance.data::<Manager>();
        data.attribute_names
            .insert(&data.attribute_count, &attribute_input);
    }
}

/// Helper trait for Psp34Trait
impl<T> InternalTraits for T
where
    T: Storage<Manager> + Storage<psp34::Data<enumerable::Balances>>,
{
    /// Check if the transferred mint values is as expected
    default fn check_value(&self, transferred_value: u128, mint_amount: u64) -> Result<(), Error> {
        if let Some(value) =
            (mint_amount as u128).checked_mul(self.data::<Manager>().price_per_mint)
        {
            if transferred_value == value {
                return Ok(());
            }
        }
        return Err(Error::Custom(String::from("Bad Mint Value")));
    }

    /// Check amount of tokens to be minted
    default fn check_amount(&self, mint_amount: u64) -> Result<(), Error> {
        if mint_amount == 0 {
            return Err(Error::Custom(String::from("Cannot Mint Zero Tokens")));
        }
        if mint_amount > self.data::<Manager>().max_amount {
            return Err(Error::Custom(String::from("Too Many Tokens To Mint")));
        }
        if let Some(amount) = self
            .data::<Manager>()
            .last_token_id
            .checked_add(mint_amount)
        {
            if amount <= self.data::<Manager>().max_supply {
                return Ok(());
            }
        }
        return Err(Error::Custom(String::from("Collection Is Full")));
    }

    /// Check if token is minted
    default fn token_exists(&self, id: Id) -> Result<(), Error> {
        self.data::<psp34::Data<enumerable::Balances>>()
            .owner_of(id)
            .ok_or(Error::BidNotExist)?;
        Ok(())
    }
}
