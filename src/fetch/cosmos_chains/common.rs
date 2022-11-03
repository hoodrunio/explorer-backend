/// The struct that stores important URLs of a chain.
pub struct ChainUrls {
    rest_api: &'static str,
}

/** The trait that provides common methods of chains. */
pub trait Chain {
    /** Returns the name of the chain. */
    fn get_name() -> &'static str;
    /** Returns the `ChainUrls` of the chain. */
    fn get_urls() -> ChainUrls;
}
