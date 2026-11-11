//! Tickets use cases: who may do what, on top of the domain rules and its ports.

pub mod app_error;
pub mod ticket_page;
pub mod ticket_service;

pub use app_error::AppError;
pub use ticket_page::TicketPage;
pub use ticket_service::TicketService;
