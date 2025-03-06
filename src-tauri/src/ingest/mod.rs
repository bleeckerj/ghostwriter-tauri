pub mod document_ingestor;
pub use document_ingestor::*;

pub mod pdf_ingestor;
pub mod mdx_ingestor;
pub mod markdown_ingestor;
pub mod epub_ingestor;
pub mod text_ingestor;
pub mod url_ingestor;
pub mod mongodb_ingestor;

pub use pdf_ingestor::PdfIngestor;
pub use mdx_ingestor::MdxIngestor; 
pub use markdown_ingestor::MarkdownIngestor;
pub use epub_ingestor::EpubIngestor;
pub use text_ingestor::TextIngestor;
pub use url_ingestor::UrlDocumentIngestor;
pub use document_ingestor::{DocumentIngestor, Resource, IngestedDocument, DocumentMetadata, IngestError};
pub use mongodb_ingestor::{MongoDocumentIngestor, MongoConfig};