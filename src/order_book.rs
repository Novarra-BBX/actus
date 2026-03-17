// Copyright (C) 2019-2025 Alpha-Delta Network Inc.
// This file is part of the actus library.

// The actus library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The actus library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the actus library. If not, see <https://www.gnu.org/licenses/>.

/// Method used to determine the ask price for a listed position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PriceDiscoveryMethod {
    GovernorOracle,
    ZkNpv,
    ManualAsk,
}

/// A listing on the DEX order book.
#[derive(Debug, Clone)]
pub struct Listing {
    pub listing_id: u64,
    pub contract_id: String,
    pub seller: [u8; 32],
    /// Ask price in sAX (18-decimal fixed-point units).
    pub ask_price_sax: u128,
    pub price_method: PriceDiscoveryMethod,
    pub expiry_block: u64,
    pub is_active: bool,
}

/// A bid placed against a listing.
#[derive(Debug, Clone)]
pub struct Bid {
    pub bid_id: u64,
    pub listing_id: u64,
    pub buyer: [u8; 32],
    pub bid_price_sax: u128,
    pub expiry_block: u64,
}

/// DEX order book managing listings and bids for tokenized ACTUS positions.
pub struct OrderBook {
    pub listings: Vec<Listing>,
    pub bids: Vec<Bid>,
    next_listing_id: u64,
    next_bid_id: u64,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            listings: Vec::new(),
            bids: Vec::new(),
            next_listing_id: 1,
            next_bid_id: 1,
        }
    }

    /// List a tokenized position for sale. Returns the assigned `listing_id`.
    pub fn list_position(
        &mut self,
        contract_id: impl Into<String>,
        seller: [u8; 32],
        ask_price_sax: u128,
        price_method: PriceDiscoveryMethod,
        expiry_block: u64,
    ) -> u64 {
        let listing_id = self.next_listing_id;
        self.next_listing_id += 1;
        self.listings.push(Listing {
            listing_id,
            contract_id: contract_id.into(),
            seller,
            ask_price_sax,
            price_method,
            expiry_block,
            is_active: true,
        });
        listing_id
    }

    /// Place a bid on an active listing. Returns `bid_id` or `Err` if listing not found / inactive.
    pub fn place_bid(
        &mut self,
        listing_id: u64,
        buyer: [u8; 32],
        bid_price_sax: u128,
        expiry_block: u64,
    ) -> Result<u64, String> {
        let listing = self
            .listings
            .iter()
            .find(|l| l.listing_id == listing_id)
            .ok_or_else(|| format!("Listing {} not found", listing_id))?;

        if !listing.is_active {
            return Err(format!("Listing {} is not active", listing_id));
        }

        let bid_id = self.next_bid_id;
        self.next_bid_id += 1;
        self.bids.push(Bid {
            bid_id,
            listing_id,
            buyer,
            bid_price_sax,
            expiry_block,
        });
        Ok(bid_id)
    }

    /// Fill a listing by selecting the highest non-expired bid. Deactivates the listing.
    /// Returns `(buyer, price)` or `Err` if no valid bids exist or listing is inactive.
    pub fn fill_order(
        &mut self,
        listing_id: u64,
        current_block: u64,
    ) -> Result<([u8; 32], u128), String> {
        // Verify listing is active
        let listing = self
            .listings
            .iter()
            .find(|l| l.listing_id == listing_id)
            .ok_or_else(|| format!("Listing {} not found", listing_id))?;

        if !listing.is_active {
            return Err(format!("Listing {} is not active", listing_id));
        }

        // Find best (highest price) non-expired bid for this listing
        let best_bid = self
            .bids
            .iter()
            .filter(|b| b.listing_id == listing_id && current_block < b.expiry_block)
            .max_by_key(|b| b.bid_price_sax)
            .map(|b| (b.buyer, b.bid_price_sax));

        let (buyer, price) = best_bid.ok_or_else(|| {
            format!("No valid bids for listing {}", listing_id)
        })?;

        // Deactivate listing
        if let Some(l) = self.listings.iter_mut().find(|l| l.listing_id == listing_id) {
            l.is_active = false;
        }

        Ok((buyer, price))
    }

    /// Cancel a listing. Only the original seller may cancel.
    pub fn cancel_listing(&mut self, listing_id: u64, caller: [u8; 32]) -> Result<(), String> {
        let listing = self
            .listings
            .iter_mut()
            .find(|l| l.listing_id == listing_id)
            .ok_or_else(|| format!("Listing {} not found", listing_id))?;

        if listing.seller != caller {
            return Err("Unauthorized: caller is not the seller".to_string());
        }

        listing.is_active = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seller() -> [u8; 32] { [1u8; 32] }
    fn buyer() -> [u8; 32] { [2u8; 32] }

    #[test]
    fn test_list_and_fill() {
        let mut ob = OrderBook::new();
        let lid = ob.list_position(
            "PAM-001",
            seller(),
            1_000_000_000_000_000_000u128,
            PriceDiscoveryMethod::ZkNpv,
            500,
        );

        let _bid_id = ob
            .place_bid(lid, buyer(), 950_000_000_000_000_000u128, 500)
            .expect("place_bid should succeed");

        let (filled_buyer, filled_price) = ob.fill_order(lid, 100).expect("fill_order should succeed");

        assert_eq!(filled_buyer, buyer());
        assert_eq!(filled_price, 950_000_000_000_000_000u128);

        // Listing should now be inactive
        let listing = ob.listings.iter().find(|l| l.listing_id == lid).unwrap();
        assert!(!listing.is_active);
    }

    #[test]
    fn test_fill_expired_bid() {
        let mut ob = OrderBook::new();
        let lid = ob.list_position(
            "LAM-002",
            seller(),
            500_000u128,
            PriceDiscoveryMethod::ManualAsk,
            1000,
        );

        // expiry_block = 0, current_block = 100 → bid is expired (current_block >= expiry_block)
        ob.place_bid(lid, buyer(), 499_000u128, 0)
            .expect("place_bid should succeed");

        let result = ob.fill_order(lid, 100);
        assert!(result.is_err(), "fill_order should fail when only expired bids exist");
    }

    #[test]
    fn test_cancel_listing() {
        let mut ob = OrderBook::new();
        let lid = ob.list_position(
            "STK-003",
            seller(),
            100u128,
            PriceDiscoveryMethod::GovernorOracle,
            200,
        );

        ob.cancel_listing(lid, seller()).expect("cancel by seller should succeed");

        let listing = ob.listings.iter().find(|l| l.listing_id == lid).unwrap();
        assert!(!listing.is_active);
    }

    #[test]
    fn test_unauthorized_cancel() {
        let mut ob = OrderBook::new();
        let lid = ob.list_position(
            "ANN-004",
            seller(),
            200u128,
            PriceDiscoveryMethod::GovernorOracle,
            300,
        );

        let result = ob.cancel_listing(lid, buyer());
        assert!(result.is_err(), "cancel by non-seller should return Err");
    }
}
