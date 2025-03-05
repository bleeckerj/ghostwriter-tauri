use sodiumoxide::crypto::box_;
use keyring::Entry;
use std::error::Error;



pub struct KeychainHandler;


impl KeychainHandler {
    
    const SERVICE_NAME: &str = "com.nearfuturelaboratory.ghostwriter";
    const API_KEY_NAME: &str = "openai_api_key";

    /// Store the OpenAI API key securely in the OS keyring
    pub fn store_api_key(api_key: &str) -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(Self::SERVICE_NAME, Self::API_KEY_NAME)?;
        println!("Entry is {:?}", entry);
        match entry.set_password(api_key) {
            Ok(_) => (),
            Err(e) => {
                println!("❌ Failed to store OpenAI API Key
                securely in the keyring: {}", e);
                log::error!("❌ Failed to store OpenAI API Key securely in the keyring: {}", e);
                return Err(e.into());
            }
        }
        // println!("✅ OpenAI API Key stored securely in the keyring.");
        // log::debug!("OpenAI API Key stored securely in the keyring.");
        Ok(())
    }
    
    /// Retrieve the OpenAI API key securely from the OS keyring
    pub fn retrieve_api_key() -> Result<Option<String>, Box<dyn Error>> {
        let entry = Entry::new(Self::SERVICE_NAME, Self::API_KEY_NAME)?;
        match entry.get_password() {
            Ok(api_key) => Ok(Some(api_key)),
            Err(_) => {
                println!("⚠️ OpenAI API Key not found in the keyring.");
                log::error!("OpenAI API Key not found in the keyring.");

                Ok(None)
            }
        }
    }
    
    /// Delete the OpenAI API key from the OS keyring
    pub fn delete_api_key() -> Result<(), Box<dyn Error>> {
        let entry = Entry::new(Self::SERVICE_NAME, Self::API_KEY_NAME)?;
        entry.delete_credential()?;
        println!("🗑️ OpenAI API Key deleted from the keyring.");
        Ok(())
    }
}