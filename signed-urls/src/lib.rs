use failure::Error;
use std::collections::BTreeMap;
use std::time;

struct QueryParams {
    map: BTreeMap<String, String>,
}

pub fn sign_url(secret: &str, url: &str, expiration: time::SystemTime) -> Result<String, Error> {
    unimplemented!()
}

pub fn validate(secret: &str, url: &str) -> Result<(), Error> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
