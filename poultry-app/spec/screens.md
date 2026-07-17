# Poultry Farm Application - Screens Specification

This document details the UI layout and screen flow for the Dioxus-based web/desktop application. The application will use a modern, responsive sidebar layout.

## 1. Main Dashboard (`/`)

The landing screen for authenticated users. Provides a high-level overview of the farm's daily and monthly performance.
- **Top Metric Cards:**
  - Today's Egg Sales (Revenue & Volume)
  - Today's Broken Egg Sales
  - Today's Material Purchases (Expenses)
  - Today's Labor Attendance
- **Charts / Visuals:**
  - 7-Day Trend: Revenue vs Expenses
  - Feed Production Trend
- **Alerts / Notifications:**
  - High Outstanding Balances (Customers who owe > X limit)
  - Low Material Inventory (if tracked)

## 2. Sales Module (`/sales`)

A tabbed interface to separate standard and broken egg sales.

### Tab 1: Standard Eggs
- **Data Table:** Shows recent sales (Date, Customer, Boxes, Rate, Amount, Balance).
- **Actions:** 'Add Sale' button opening a slide-out drawer or modal.
- **Add Sale Form:**
  - Select Customer (Searchable dropdown)
  - Date Picker (Defaults to today)
  - Inputs: Boxes, Size, Gross Rate, Less (Discount)
  - Auto-calculated: Total Eggs, Net Rate, Total Amount
  - Inputs: Received Amount, Payment Mode

### Tab 2: Broken Eggs
- **Data Table:** Shows Date, Bakery Name, Trays Sold, Amount, Return Trays.
- **Actions:** 'Add Broken Sale' / 'Log Returned Trays'.

## 3. Purchases Module (`/purchases`)

- **Data Table:** Shows Date, Supplier, Material, Weight, Amount, Paid Status.
- **Actions:** 'Add Purchase' button.
- **Add Purchase Form:**
  - Select Supplier
  - Material Name (Dropdown with standard items like Maize, Soya, etc., or add new)
  - Inputs: Weight, Unit Rate, Advance Paid.
  - Auto-calculated: Total Amount, Remaining Balance.

## 4. Feed Management (`/feed`)

- **Data Table:** Shows Date, Batch ID, Feed Type, Rate, Total Amount.
- **Actions:** 'Log New Batch'.
- **Metrics Bar:** Opening and Closing Balances for the current month.

## 5. Labor Management (`/labor`)

- **Daily Attendance View:**
  - A grid showing Employees as rows and Days of the month as columns.
  - Clicking a cell toggles attendance (Present `P`, Half-day `H`, Absent `A`).
- **Advances & Wages Tab:**
  - A table showing each employee's accumulated working days, calculated wage, total advances taken, and net payout pending.
- **Actions:** 'Give Advance', 'Process Payout'.

## 6. Parties & Ledger (`/parties`)

- **Party Directory:**
  - A list of all entities grouped by Type (Customer, Supplier, Employee).
  - Shows current running balance (Green for they owe us, Red for we owe them).
- **Party Detail View (`/parties/:id`):**
  - Displays contact details.
  - A chronological ledger table showing all transactions (Sales, Purchases, Advances, Payments) and the running balance after each transaction.
