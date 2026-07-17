# Poultry Farm Application - Features Specification

This document outlines the core business logic, modules, and data structures required for the Farmiz Poultry Application, derived from the provided Excel/HTML tracking data.

## 1. Egg Sales Management

The Egg Sales module is split into two primary areas: Standard Egg Sales and Broken Egg Sales.

### 1.1 Standard Egg Sales
Records bulk sales of standard eggs to large customers/distributors.
- **Fields to Track:**
  - `Date` (DD.MM.YY)
  - `Party` (Customer Name, e.g., AKG, MALIKA)
  - `Quantity` (Number of Boxes/Trays)
  - `Total Eggs` (Usually Quantity * 30, but explicitly tracked)
  - `Size` (e.g., LARGE, MEDIUM)
  - `Gross Rate` (Price per egg)
  - `Less / Discount` (Brokerage/discount per egg)
  - `Net Rate` (Gross Rate - Less)
  - `Total Amount` (Total Eggs * Net Rate)
  - `Received Amount / Paid`
  - `Payment Mode` (CASH / UPI / BANK)
  - `Balance` (Running balance for the party)

### 1.2 Broken Egg Sales
Records sales of broken or lower-grade eggs, primarily to local bakeries (e.g., SURIYA BAKERY, IYANGAR).
- **Fields to Track:**
  - `Date`
  - `Party` (Bakery Name)
  - `Trays Sold`
  - `Rate` (Per tray or egg)
  - `Amount`
  - `Payment Received`
  - `Return Trays` (Trays brought back by the customer)
  - `Empty Trays Balance` (Running balance of unreturned trays)
  - `Balance Amount`

## 2. Material Purchases & Inventory

Tracks all purchases made by the farm for raw materials (Maize, Soya, medicines like Kemin Keprex, etc.).
- **Fields to Track:**
  - `Date`
  - `Material Name` (e.g., Maize, Soya, LYSINE, DLM)
  - `Party` (Supplier/Vendor Name, e.g., Madesh, Silambarasan)
  - `Weight / Quantity` (in kg or units)
  - `1KG Rate` (Unit price)
  - `Total Amount`
  - `Advance Paid`
  - `Status` (PAID / PENDING)
  - `Balance` (Running balance payable to supplier)

## 3. Feed Mill Data

Tracks feed production and usage within the farm.
- **Fields to Track:**
  - `Date`
  - `Batch ID`
  - `Feed Type`
  - `Rate`
  - `Total Amount`
  - `Payment` (if applicable)
  - `Opening Balance` (Financial or Inventory)
  - `Closing Balance`
  - `Total Batch Volume`

## 4. Labor Management

Tracks employee attendance and wage payouts.
- **Fields to Track:**
  - `Date`
  - `Employee Name` (e.g., RAJENDER, BEHAN)
  - `Attendance` (1 for full day, 0.5 for half day, 0 for absent)
  - `Advance Given` (Cash paid out early)
- **Calculated Metrics (per period/month):**
  - `Total Working Days`
  - `Working Amount` (Wages calculated based on working days)
  - `Total Advance` (Sum of all advances)
  - `Balance Amount` (Working Amount - Total Advance)

## 5. Party Ledger (Unified Directory)

A centralized system to manage entities the farm interacts with.
- **Party Types:** Customer, Supplier, Employee, Bakery.
- **Ledger Logic:** Every transaction from Sales, Purchases, or Labor should post to the Party's ledger to maintain a unified running balance (Accounts Receivable vs Accounts Payable).
