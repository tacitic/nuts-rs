use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use failure::Error;
use std::collections::BTreeMap;
use std::time;
use std::time::SystemTime;
use url::Url;

#[macro_use]
extern crate failure;

const EXPIRATION_QUERY_PARAM: &str = "exp";
const SIGNATURE_QUERY_PARAM: &str = "signature";

pub fn sign_url(secret: &str, url: &str, exp: time::SystemTime) -> Result<String, Error> {
    let url = Url::parse(url)?;

    // Extract query params
    let mut query_params = BTreeMap::new();

    for (k, v) in url.query_pairs() {
        query_params.insert(k.to_string(), v.to_string());
    }

    // Add expiration time
    let timestamp = exp.duration_since(SystemTime::UNIX_EPOCH)?;
    query_params.insert(
        EXPIRATION_QUERY_PARAM.to_string(),
        timestamp.as_secs().to_string(),
    );

    // Create temporary url for siging only. this is not exposed back to the user.
    let mut url_for_signing = url.clone();
    url_for_signing
        .query_pairs_mut()
        .clear()
        .extend_pairs(query_params);

    let signature = get_signature(&url_for_signing, &secret)?;

    let mut signed_url = url.clone();
    signed_url
        .query_pairs_mut()
        .append_pair(
            EXPIRATION_QUERY_PARAM,
            timestamp.as_secs().to_string().as_str(),
        )
        .append_pair(SIGNATURE_QUERY_PARAM, signature.as_str());

    Ok(signed_url.to_string())
}

pub fn validate(secret: &str, url: &str) -> Result<(), Error> {
    let url = Url::parse(url)?;

    // Extract query params
    let mut query_params = BTreeMap::new();
    let mut url_signature: Option<String> = None;
    let mut exp: Option<String> = None;

    for (k, v) in url.query_pairs() {
        if k == SIGNATURE_QUERY_PARAM {
            url_signature = Some(v.to_string());
        } else if k == EXPIRATION_QUERY_PARAM {
            exp = Some(v.to_string());
            query_params.insert(k.to_string(), v.to_string());
        } else {
            query_params.insert(k.to_string(), v.to_string());
        }
    }

    // Check exp
    if let Some(e) = exp {
        let now = time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let exp_secs: u64 = e.parse()?;
        if exp_secs <= now.as_secs() {
            bail!("Url expired");
        }
    } else {
        bail!("No expiration found signature");
    }

    // Create temporary url for siging only. this is not exposed back to the user.
    let mut url_for_signing = url.clone();
    url_for_signing
        .query_pairs_mut()
        .clear()
        .extend_pairs(query_params);

    let signature = get_signature(&url_for_signing, &secret)?;

    if let Some(s) = url_signature {
        if s != signature {
            bail!("Invalid signature");
        }
    } else {
        bail!("No signature");
    }

    Ok(())
}

fn get_signature(url: &Url, secret: &str) -> Result<String, Error> {
    let mut hmac = Hmac::new(Sha256::new(), secret.as_bytes());
    hmac.input(url.as_str().as_bytes());
    let res = hmac.result();
    Ok(base32::encode(base32::Alphabet::Crockford, res.code()).to_lowercase())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_sign_url() {
        let exp = time::SystemTime::now() + time::Duration::from_secs(600);
        let signed = sign_url(
            "supersecret",
            "https://api.github.com/x/y/asset?x=1&b=2&a=3",
            exp,
        )
        .unwrap();
        validate("supersecret", signed.as_str()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_sign_url_expired() {
        let exp = time::SystemTime::now() + time::Duration::from_secs(0);
        let signed = sign_url(
            "supersecret",
            "https://api.github.com/x/y/asset?x=1&b=2&a=3",
            exp,
        )
        .unwrap();
        validate("supersecret", signed.as_str()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_sign_url_invalid_secret() {
        let exp = time::SystemTime::now() + time::Duration::from_secs(0);
        let signed = sign_url(
            "supersecret",
            "https://api.github.com/x/y/asset?x=1&b=2&a=3",
            exp,
        )
        .unwrap();
        validate("nososecret", signed.as_str()).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_sign_url_modified() {
        let exp = time::SystemTime::now() + time::Duration::from_secs(0);
        let signed = sign_url(
            "supersecret",
            "https://api.github.com/x/y/asset?x=1&b=2&a=3",
            exp,
        )
        .unwrap();

        let signed_modified = signed + "&foo=bar";
        validate("nososecret", signed_modified.as_str()).unwrap();
    }
}
