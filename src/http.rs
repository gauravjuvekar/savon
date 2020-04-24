use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_xml_rs::{from_str, to_string};
use crate::gen::{FromElement, ToElements};
use crate::rpser::{Method, Response};
use std::fmt::Debug;

pub async fn one_way<Input: ToElements>(client: &Client, base_url: &str, ns: &str, method: &str, input: &Input) -> Result<(), crate::Error> {
    let mut v = input.to_elements();
    let mut m = Method::new(method);

    for el in v.drain(..) {
        m = m.with(el);
    }
    let s = m.as_xml(ns);
    trace!("sending: {}", s);

    let response: String = client.post(base_url)
        .header("Content-Type", "text/xml")
        .header("MessageType", "Call")
        .body(s)
        .send()
        .await?.text().await?;

    println!("received: {}", response);
    Ok(())
}

pub async fn request_response<'a, Input: ToElements, Output: Debug + FromElement + Deserialize<'a>, Error: Deserialize<'a>>(client: &Client, base_url: &str, ns: &str, method: &str, input: &Input)
    -> Result<Result<Output, Error>, crate::Error> {
    let mut v = input.to_elements();
    let mut m = Method::new(method);

    for el in v.drain(..) {
        m = m.with(el);
    }
    let s = m.as_xml(ns);
    trace!("sending: {}", s);

    let response: String = client.post(base_url)
        .header("Content-Type", "text/xml")
        .header("MessageType", "Call")
        .body(s)
        .send()
        .await?.text().await?;

    println!("received: {}", response);
    let r = Response::from_xml(&response).unwrap();
    println!("parsed: {:#?}", r);
    println!("output: {:#?}", Output::from_element(&r.body));
    let res: Result<Output, _> = from_str(&response);

    match res {
        Ok(o) => Ok(Ok(o)),
        Err(e) => match from_str(&response) {
            Err(e) => Err(e.into()),
            Ok(e) => Ok(Err(e)),
        }
    }
}
