use serde::{Deserialize, Serialize};

pub mod admin;
pub mod camera;
pub mod poi;
pub mod facetrack;
pub mod cartrack;
pub mod coi;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPage {
    #[serde(rename = "Total")]
    pub total: u64,

    #[serde(rename = "PageSize")]
    pub page_size: u64,

    #[serde(rename = "PageNumber")]
    pub page_number: u64,
}

impl DataPage {
    fn calc_pages(total: u64, page_size: u64) -> u64 {
        // page_size 不为 0
        let mut num = total / page_size;
        if total % page_size != 0 {
            num += 1;
        }
        num
    }

    pub fn new(total: u64, page_size: u64, page_number: u64) -> Self {
        let total_page = Self::calc_pages(total, page_size);
        let mut pn = page_number;
        if page_number > total_page {
            pn = 1;
        }
        DataPage {
            total,
            page_size,
            page_number: pn,
        }
    }

    pub fn get_start_index(&self) -> u64 {
        (self.page_number - 1) * self.page_size
    }

    pub fn get_total_page(&self) -> u64 {
        Self::calc_pages(self.total, self.page_size)
    }
}
