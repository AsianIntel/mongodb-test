use hyper::{
    body::{self, Buf},
    client::HttpConnector,
    Body,
    Client as HyperClient,
    Error as HyperError,
    Method,
    Request,
    Response,
};
use serde::Deserialize;
use serde_json::Error as SerdeError;

#[derive(Clone, Debug, Default)]
pub(crate) struct HttpClient {
    inner: HyperClient<HttpConnector>,
}

#[derive(Debug)]
pub(crate) enum HttpError {
    BuildingRequest,
    Request(HyperError),
    InvalidUTF8,
    Parsing(SerdeError),
}

impl HttpClient {
    /// Executes an HTTP GET request and deserializes the JSON response.
    pub(crate) async fn get_and_deserialize_json<'a, T>(
        &self,
        uri: &str,
        headers: impl IntoIterator<Item = &'a (&'a str, &'a str)>,
    ) -> Result<T, HttpError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let res = self.request(Method::GET, uri, headers).await?;

        let mut buf = body::aggregate(res.into_body()).await?;
        let mut bytes = vec![0; buf.remaining()];
        buf.copy_to_slice(&mut bytes);

        let result = serde_json::from_slice(&bytes)?;
        Ok(result)
    }

    /// Executes an HTTP GET request and returns the response body as a string.
    pub(crate) async fn get_and_read_string<'a>(
        &self,
        uri: &str,
        headers: impl IntoIterator<Item = &'a (&'a str, &'a str)>,
    ) -> Result<String, HttpError> {
        self.request_and_read_string(Method::GET, uri, headers)
            .await
    }

    /// Executes an HTTP PUT request and returns the response body as a string.
    pub(crate) async fn put_and_read_string<'a>(
        &self,
        uri: &str,
        headers: impl IntoIterator<Item = &'a (&'a str, &'a str)>,
    ) -> Result<String, HttpError> {
        self.request_and_read_string(Method::PUT, uri, headers)
            .await
    }

    /// Executes an HTTP request and returns the response body as a string.
    pub(crate) async fn request_and_read_string<'a>(
        &self,
        method: Method,
        uri: &str,
        headers: impl IntoIterator<Item = &'a (&'a str, &'a str)>,
    ) -> Result<String, HttpError> {
        let res = self.request(method, uri, headers).await?;

        let mut buf = body::aggregate(res.into_body()).await?;
        let mut bytes = vec![0; buf.remaining()];
        buf.copy_to_slice(&mut bytes);

        let text = String::from_utf8(bytes)?;
        Ok(text)
    }

    /// Executes an HTTP equest and returns the response.
    pub(crate) async fn request<'a>(
        &self,
        method: Method,
        uri: &str,
        headers: impl IntoIterator<Item = &'a (&'a str, &'a str)>,
    ) -> Result<Response<Body>, HttpError> {
        let mut request = Request::builder().uri(uri).method(method);

        for header in headers {
            request = request.header(header.0, header.1);
        }

        let request = request.body(Body::empty()).unwrap();
        let response = self.inner.request(request).await?;

        Ok(response)
    }
}

impl From<hyper::http::Error> for HttpError {
    fn from(_err: hyper::http::Error) -> Self {
        Self::BuildingRequest
    }
}

impl From<HyperError> for HttpError {
    fn from(err: HyperError) -> Self {
        Self::Request(err)
    }
}

impl From<SerdeError> for HttpError {
    fn from(err: SerdeError) -> Self {
        Self::Parsing(err)
    }
}

impl From<std::string::FromUtf8Error> for HttpError {
    fn from(_err: std::string::FromUtf8Error) -> Self {
        Self::InvalidUTF8
    }
}