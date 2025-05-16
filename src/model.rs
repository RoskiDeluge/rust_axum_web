use crate::{ctx::Ctx, Error, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// region: --- Ticket Types

#[derive(Clone, Debug, Serialize)]
pub struct Ticket {
    pub id: u64,
    pub cid: u64, // creator user_id
    pub title: String,
}

#[derive(Deserialize)]
pub struct TicketForCreate {
    pub title: String,
}

#[derive(Clone)]
pub struct ModelController {
    tickets_store: Arc<Mutex<Vec<Option<Ticket>>>>,
}

// Constructor
impl ModelController {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            tickets_store: Arc::default(),
        })
    }
}

// CRUD Implementation
impl ModelController {
    pub async fn create_ticket(&self, ctx: Ctx, ticket_fc: TicketForCreate) -> Result<Ticket> {
        let mut store: std::sync::MutexGuard<'_, Vec<Option<Ticket>>> =
            self.tickets_store.lock().unwrap();

        let id: u64 = store.len() as u64;
        let ticket: Ticket = Ticket {
            id,
            cid: ctx.user_id(),
            title: ticket_fc.title,
        };
        store.push(Some(ticket.clone()));

        Ok(ticket)
    }

    pub async fn list_tickets(&self, _ctx: Ctx) -> Result<Vec<Ticket>> {
        let store: std::sync::MutexGuard<'_, Vec<Option<Ticket>>> =
            self.tickets_store.lock().unwrap();

        let tickets: Vec<Ticket> = store
            .iter()
            .filter_map(|t: &Option<Ticket>| t.clone())
            .collect();

        Ok(tickets)
    }

    pub async fn delete_ticket(&self, _ctx: Ctx, id: u64) -> Result<Ticket> {
        let mut store: std::sync::MutexGuard<'_, Vec<Option<Ticket>>> =
            self.tickets_store.lock().unwrap();

        let ticket: Option<Ticket> = store
            .get_mut(id as usize)
            .and_then(|t: &mut Option<Ticket>| t.take());

        ticket.ok_or(Error::TicketDeleteFailIdNotFound { id })
    }
}
