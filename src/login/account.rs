use std::collections::HashMap;
use std::string::String;

use net::ClientId;
use ship::ShipStored;
use sector_data::SectorId;

pub type AccountBox = Box<Account>;

#[derive(PartialEq)]
pub enum LoginError {
    NoSuchAccount,
    WrongPassword,
    AlreadyLoggedIn,
}

pub struct Account {
    pub username: String,
    pub password: String,
    pub ship: Option<ShipStored>,
    pub client_id: Option<ClientId>,
    pub sector: SectorId,
}

pub struct AccountManager {
    accounts: HashMap<String, Option<AccountBox>>,
}

impl AccountManager {
    pub fn new() -> AccountManager {
        AccountManager {
            accounts: HashMap::new(),
        }
    }
    
    /// Creates a new account with no ship and no client ID
    pub fn create_account(&mut self, username: String, password: String) {
        self.accounts.insert(username.clone(), Some(Box::new(Account {
            username: username,
            password: password,
            ship: None,
            client_id: None,
            sector: SectorId(0),
        })));
    }
    
    /// Attempts to log an account in and returns the AccountBox on success.
    /// If the login fails, the corresponding error is returned.
    pub fn login_account(&mut self, username: String, password: String, client_id: ClientId) -> Result<AccountBox, LoginError> {
        use std::collections::hash_map::Entry;
    
        if let Entry::Occupied(mut account_entry) = self.accounts.entry(username) {
            // Make sure the account is available to log into
            if account_entry.get().is_none() {
                // This account is set to None, which means it's already logged in.
                Err(LoginError::AlreadyLoggedIn)
            } else {
                // Verify password
                if account_entry.get().as_ref().unwrap().password == password {
                    // All good, log the account in
                    
                    // Set the client ID
                    account_entry.get_mut().as_mut().unwrap().client_id = Some(client_id);
                    
                    // Remove the account and replace it with None to show the account is logged in.
                    Ok(account_entry.insert(None).unwrap())
                } else {
                    Err(LoginError::WrongPassword)
                }
            }
        } else {
            Err(LoginError::NoSuchAccount)
        }
    }
}