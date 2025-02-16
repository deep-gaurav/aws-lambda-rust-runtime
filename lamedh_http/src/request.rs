//! ALB and API Gateway request adaptations
//!
//! Typically these are exposed via the `request_context`
//! request extension method provided by [lambda_http::RequestExt](../trait.RequestExt.html)
//!
use crate::{
    ext::{PathParameters, QueryStringParameters, StageVariables},
    strmap::StrMap,
};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::event::alb::{AlbTargetGroupRequest, AlbTargetGroupRequestContext};
use aws_lambda_events::event::apigw::{
    ApiGatewayProxyRequest, ApiGatewayProxyRequestContext, ApiGatewayV2httpRequest, ApiGatewayV2httpRequestContext,
};
use http::{HeaderMap, HeaderValue, Method, header::HeaderName};
use serde::Deserialize;
use serde_json::error::Error as JsonError;
use std::{io::Read, mem};

/// Internal representation of an Lambda http event from
/// ALB, API Gateway REST and HTTP API proxy event perspectives
///
/// This is not intended to be a type consumed by crate users directly. The order
/// of the variants are notable. Serde will try to deserialize in this order.
#[doc(hidden)]
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum LambdaRequest {
    ApiGatewayV1(ApiGatewayProxyRequest),
    ApiGatewayV2(ApiGatewayV2httpRequest),
    Alb(AlbTargetGroupRequest),
    Unknown(FixReq),
}
/// FIx for vercel
#[doc(hidden)]
#[derive(Debug,Deserialize)]
pub struct FixReq{
    #[serde(with = "http_serde::method")]
    method:Method,
    // host:String,
    path:Option<String>,
    #[serde(with = "http_serde::header_map",default)]
    headers:HeaderMap<HeaderValue>,
    
    #[serde(default)]
    encoding:Option<String>,
    #[serde(default)]
    body:Option<String>
}


// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub struct ApiGatewayProxyRequest2 {
//     /// The resource path defined in API Gateway
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     pub resource: Option<String>,
//     /// The url path for the caller
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     pub path: Option<String>,
//     #[serde(with = "http_method")]
//     #[serde(rename = "httpMethod")]
//     pub http_method: Method,
//     #[serde(deserialize_with = "http_serde::header_map::deserialize")]
//     #[serde(serialize_with = "serialize_headers")]
//     pub headers: HeaderMap,
//     #[serde(deserialize_with = "http_serde::header_map::deserialize")]
//     #[serde(serialize_with = "serialize_multi_value_headers")]
//     #[serde(rename = "multiValueHeaders")]
//     pub multi_value_headers: HeaderMap,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "queryStringParameters")]
//     pub query_string_parameters: HashMap<String, String>,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "multiValueQueryStringParameters")]
//     pub multi_value_query_string_parameters: HashMap<String, Vec<String>>,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "pathParameters")]
//     pub path_parameters: HashMap<String, String>,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "stageVariables")]
//     pub stage_variables: HashMap<String, String>,
//     #[serde(default)]
//     #[serde(rename = "requestContext")]
//     pub request_context: ApiGatewayProxyRequestContext,
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     pub body: Option<String>,
//     #[serde(skip_serializing_if = "Option::is_none")]
//     #[serde(rename = "isBase64Encoded")]
//     pub is_base64_encoded: Option<bool>,
// }


// /// `ApiGatewayV2httpRequest` contains data coming from the new HTTP API Gateway
// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub struct ApiGatewayV2httpRequest2 {
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     pub version: Option<String>,
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     #[serde(rename = "routeKey")]
//     pub route_key: Option<String>,
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     #[serde(rename = "rawPath")]
//     pub raw_path: Option<String>,
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     #[serde(rename = "rawQueryString")]
//     pub raw_query_string: Option<String>,
//     pub cookies: Option<Vec<String>>,
//     #[serde(deserialize_with = "http_serde::header_map::deserialize")]
//     #[serde(serialize_with = "serialize_headers")]
//     pub headers: HeaderMap,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "queryStringParameters")]
//     pub query_string_parameters: HashMap<String, String>,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "pathParameters")]
//     pub path_parameters: HashMap<String, String>,
//     #[serde(rename = "requestContext")]
//     pub request_context: ApiGatewayV2httpRequestContext,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "stageVariables")]
//     pub stage_variables: HashMap<String, String>,
//     pub body: Option<String>,
//     #[serde(rename = "isBase64Encoded")]
//     pub is_base64_encoded: bool,
// }


// /// `AlbTargetGroupRequest` contains data originating from the ALB Lambda target group integration
// #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
// pub struct AlbTargetGroupRequest2 {
//     #[serde(with = "http_method")]
//     #[serde(rename = "httpMethod")]
//     pub http_method: Method,
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     pub path: Option<String>,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "queryStringParameters")]
//     pub query_string_parameters: HashMap<String, String>,
//     #[serde(deserialize_with = "deserialize_lambda_map")]
//     #[serde(default)]
//     #[serde(rename = "multiValueQueryStringParameters")]
//     pub multi_value_query_string_parameters: HashMap<String, Vec<String>>,
//     #[serde(deserialize_with = "http_serde::header_map::deserialize")]
//     #[serde(serialize_with = "serialize_headers")]
//     pub headers: HeaderMap,
//     #[serde(deserialize_with = "http_serde::header_map::deserialize")]
//     #[serde(serialize_with = "serialize_multi_value_headers")]
//     #[serde(rename = "multiValueHeaders")]
//     pub multi_value_headers: HeaderMap,
//     #[serde(rename = "requestContext")]
//     pub request_context: AlbTargetGroupRequestContext,
//     #[serde(rename = "isBase64Encoded")]
//     pub is_base64_encoded: bool,
//     #[serde(deserialize_with = "deserialize_lambda_string")]
//     #[serde(default)]
//     pub body: Option<String>,
// }

impl LambdaRequest {
    /// Return the `RequestOrigin` of the request to determine where the `LambdaRequest`
    /// originated from, so that the appropriate response can be selected based on what
    /// type of response the request origin expects.
    pub fn request_origin(&self) -> RequestOrigin {
        match self {
            LambdaRequest::ApiGatewayV1 { .. } => RequestOrigin::ApiGatewayV1,
            LambdaRequest::ApiGatewayV2 { .. } => RequestOrigin::ApiGatewayV2,
            LambdaRequest::Alb { .. } => RequestOrigin::Alb,
            _ => RequestOrigin::ApiGatewayV2
        }
    }
}

/// Represents the origin from which the lambda was requested from.
#[doc(hidden)]
#[derive(Debug)]
pub enum RequestOrigin {
    /// API Gateway proxy request origin
    ApiGatewayV1,
    /// API Gateway v2 request origin
    ApiGatewayV2,
    /// ALB request origin
    Alb,
}

/// Event request context as an enumeration of request contexts
/// for both ALB and API Gateway and HTTP API events
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RequestContext {
    /// API Gateway proxy request context
    ApiGatewayV1(ApiGatewayProxyRequestContext),
    /// API Gateway v2 request context
    ApiGatewayV2(ApiGatewayV2httpRequestContext),
    /// ALB request context
    Alb(AlbTargetGroupRequestContext),
}

/// Converts LambdaRequest types into `http::Request<Body>` types
impl<'a> From<LambdaRequest> for http::Request<Body> {
    fn from(value: LambdaRequest) -> Self {
        println!("lambdarequest : {:#?}",value);
        match value {
            LambdaRequest::ApiGatewayV2(ag) => into_api_gateway_v2_request(ag),
            LambdaRequest::ApiGatewayV1(ag) => into_proxy_request(ag),
            LambdaRequest::Alb(alb) => into_alb_request(alb),
            LambdaRequest::Unknown(fixreq) => {
                let http_method = fixreq.method.clone();
                let builder = http::Request::builder()
                    .uri({
                        let scheme = fixreq
                            .headers
                            .get(x_forwarded_proto())
                            .and_then(|s| s.to_str().ok())
                            .unwrap_or("https");
                        let host = fixreq
                            .headers
                            .get(http::header::HOST)
                            .and_then(|s| s.to_str().ok())
                            // .or_else(|| ag.request_context.domain_name.as_deref())
                            .unwrap_or("localhost");
            
                        let mut url = format!("{}://{}{}", scheme, host, fixreq.path.as_deref().unwrap_or_default());
                        // if let Some(query) = ag.raw_query_string {
                        //     url.push('?');
                        //     url.push_str(&query);
                        // }
                        url
                    })
                    // .extension(QueryStringParameters(StrMap::from(ag.query_string_parameters)))
                    // .extension(PathParameters(StrMap::from(ag.path_parameters)))
                    // .extension(StageVariables(StrMap::from(ag.stage_variables)))
                    // .extension(RequestContext::ApiGatewayV2(ag.request_context))
                    ;
            
                let mut headers = fixreq.headers;
                // if let Some(cookies) = ag.cookies {
                //     if let Ok(header_value) = http::header::HeaderValue::from_str(&cookies.join(";")) {
                //         headers.append(http::header::COOKIE, header_value);
                //     }
                // }
            
                let base64 = fixreq.encoding.unwrap_or_default().contains("base64");
            
                let mut req = builder
                    .body(
                        fixreq.body
                            .as_deref()
                            .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
                    )
                    .expect("failed to build request");
            
                // no builder method that sets headers in batch
                let _ = mem::replace(req.headers_mut(), headers);
                let _ = mem::replace(req.method_mut(), http_method);
            
                req
            }
        }
    }
}

pub(crate) fn into_api_gateway_v2_request(ag: ApiGatewayV2httpRequest) -> http::Request<Body> {
    let http_method = ag.request_context.http.method.clone();
    let builder = http::Request::builder()
        .uri({
            let scheme = ag
                .headers
                .get(x_forwarded_proto())
                .and_then(|s| s.to_str().ok())
                .unwrap_or("https");
            let host = ag
                .headers
                .get(http::header::HOST)
                .and_then(|s| s.to_str().ok())
                .or_else(|| ag.request_context.domain_name.as_deref())
                .unwrap_or("localhost");

            let mut url = format!("{}://{}{}", scheme, host, ag.raw_path.as_deref().unwrap_or_default());
            if let Some(query) = ag.raw_query_string {
                url.push('?');
                url.push_str(&query);
            }
            url
        })
        .extension(QueryStringParameters(StrMap::from(ag.query_string_parameters)))
        .extension(PathParameters(StrMap::from(ag.path_parameters)))
        .extension(StageVariables(StrMap::from(ag.stage_variables)))
        .extension(RequestContext::ApiGatewayV2(ag.request_context));

    let mut headers = ag.headers;
    if let Some(cookies) = ag.cookies {
        if let Ok(header_value) = http::header::HeaderValue::from_str(&cookies.join(";")) {
            headers.append(http::header::COOKIE, header_value);
        }
    }

    let base64 = ag.is_base64_encoded;

    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method);

    req
}
pub(crate) fn into_proxy_request(ag: ApiGatewayProxyRequest) -> http::Request<Body> {
    let http_method = ag.http_method;
    let builder = http::Request::builder()
        .uri({
            let scheme = ag
                .headers
                .get(x_forwarded_proto())
                .and_then(|s| s.to_str().ok())
                .unwrap_or("https");
            let host = ag
                .headers
                .get(http::header::HOST)
                .and_then(|s| s.to_str().ok())
                .unwrap_or("localhost");

            format!("{}://{}{}", scheme, host, ag.path.unwrap_or_default())
        })
        // multi-valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if ag.multi_value_query_string_parameters.is_empty() {
                StrMap::from(ag.query_string_parameters)
            } else {
                StrMap::from(ag.multi_value_query_string_parameters)
            },
        ))
        .extension(PathParameters(StrMap::from(ag.path_parameters)))
        .extension(StageVariables(StrMap::from(ag.stage_variables)))
        .extension(RequestContext::ApiGatewayV1(ag.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = ag.multi_value_headers;
    headers.extend(ag.headers);

    let base64 = ag.is_base64_encoded.unwrap_or_default();
    let mut req = builder
        .body(
            ag.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method);

    req
}

pub(crate) fn into_alb_request(alb: AlbTargetGroupRequest) -> http::Request<Body> {
    let http_method = alb.http_method;
    let builder = http::Request::builder()
        .uri({
            let scheme = alb
                .headers
                .get(x_forwarded_proto())
                .and_then(|s| s.to_str().ok())
                .unwrap_or("https");
            let host = alb
                .headers
                .get(http::header::HOST)
                .and_then(|s| s.to_str().ok())
                .unwrap_or("localhost");

            format!("{}://{}{}", scheme, host, alb.path.unwrap_or_default())
        })
        // multi valued query string parameters are always a super
        // set of singly valued query string parameters,
        // when present, multi-valued query string parameters are preferred
        .extension(QueryStringParameters(
            if alb.multi_value_query_string_parameters.is_empty() {
                StrMap::from(alb.query_string_parameters)
            } else {
                StrMap::from(alb.multi_value_query_string_parameters)
            },
        ))
        .extension(RequestContext::Alb(alb.request_context));

    // merge headers into multi_value_headers and make
    // multi-value_headers our cannoncial source of request headers
    let mut headers = alb.multi_value_headers;
    headers.extend(alb.headers);

    let base64 = alb.is_base64_encoded;

    let mut req = builder
        .body(
            alb.body
                .as_deref()
                .map_or_else(Body::default, |b| Body::from_maybe_encoded(base64, b)),
        )
        .expect("failed to build request");

    // no builder method that sets headers in batch
    let _ = mem::replace(req.headers_mut(), headers);
    let _ = mem::replace(req.method_mut(), http_method);

    req
}

/// Deserializes a `Request` from a `Read` impl providing JSON events.
///
/// # Example
///
/// ```rust,no_run
/// use lamedh_http::request::from_reader;
/// use std::fs::File;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let request = from_reader(
///         File::open("path/to/request.json")?
///     )?;
///     Ok(println!("{:#?}", request))
/// }
/// ```
pub fn from_reader<R>(rdr: R) -> Result<crate::Request, JsonError>
where
    R: Read,
{
    serde_json::from_reader(rdr).map(LambdaRequest::into)
}

/// Deserializes a `Request` from a string of JSON text.
///
/// # Example
///
/// ```rust,no_run
/// use lamedh_http::request::from_str;
/// use std::fs::File;
/// use std::error::Error;
///
/// fn main() -> Result<(), Box<dyn Error>> {
///     let request = from_str(
///         r#"{ ...raw json here... }"#
///     )?;
///     Ok(println!("{:#?}", request))
/// }
/// ```
pub fn from_str(s: &str) -> Result<crate::Request, JsonError> {
    serde_json::from_str(s).map(LambdaRequest::into)
}

fn x_forwarded_proto() -> HeaderName {
    HeaderName::from_static("x-forwarded-proto")
}
