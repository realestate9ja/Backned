use crate::interfaces::http::errors::AppError;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    page: u32,
    per_page: u32,
}

impl Pagination {
    pub fn new(page: Option<u32>, per_page: Option<u32>) -> Result<Self, AppError> {
        let page = page.unwrap_or(1);
        let per_page = per_page.unwrap_or(20);

        if page == 0 {
            return Err(AppError::bad_request("page must be greater than 0"));
        }
        if per_page == 0 || per_page > 100 {
            return Err(AppError::bad_request("per_page must be between 1 and 100"));
        }

        Ok(Self { page, per_page })
    }

    pub fn limit(self) -> i64 {
        i64::from(self.per_page)
    }

    pub fn offset(self) -> i64 {
        i64::from((self.page - 1) * self.per_page)
    }

    pub fn page(self) -> u32 {
        self.page
    }

    pub fn per_page(self) -> u32 {
        self.per_page
    }
}

impl TryFrom<PaginationParams> for Pagination {
    type Error = AppError;

    fn try_from(value: PaginationParams) -> Result<Self, Self::Error> {
        Self::new(value.page, value.per_page)
    }
}
