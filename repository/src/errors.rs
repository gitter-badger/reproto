//! Errors for repository

use core::RpPackage;
use core::errors as core;
use hyper;
use serde_json;
use std::io;
use std::time;
use toml;

error_chain! {
    links {
        Core(core::Error, core::ErrorKind);
    }

    foreign_links {
        Io(io::Error);
        TomlDe(toml::de::Error);
        SerdeJson(serde_json::Error);
        UrlParseError(::url::ParseError);
        FromHexError(::hex::FromHexError);
        HyperUriError(hyper::error::UriError);
        HyperError(hyper::Error);
        SystemTimeError(time::SystemTimeError);
    }

    errors {
        EmptyIndex {
        }

        EmptyObjects {
        }

        NoPublishIndex(url: String) {
            description("index does not support publishing")
            display("index does not support publishing: {}", url)
        }

        NoPublishObjects(url: String) {
            description("object storage does not support publishing")
            display("object storage does not support publishing: {}", url)
        }

        NoPublishedPackage(package: RpPackage) {
            description("no version published for package")
            display("no version published for package: {}", package)
        }

        PoisonError {
            description("mutex poisoned")
        }
    }
}
