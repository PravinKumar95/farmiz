import { test, expect, Page } from '@playwright/test';

// Real user credentials loaded from .env or GitHub Secrets
const e2eEmail = process.env.E2E_EMAIL?.trim();
const e2ePassword = process.env.E2E_PASSWORD?.trim();
const hasRealCredentials = Boolean(e2eEmail && e2ePassword);

// Sample mock production records
const mockProductionRecords = [
  {
    id: 'prod-rec-001',
    date: '2026-10-22',
    shed_name: 'Batch A',
    total_trays: 50.0,
    egg_count_good: 1500,
    egg_count_damaged: 2.0,
    dirty_cat1: 1.0,
    dirty_cat2: 0.5,
    production_percentage: 75.0,
    stock_in_trays: 120.0,
    mortality_count: 5,
    cull_count: 0,
    feed_consumed_kg: 0.0,
    flock_size: 2000,
    notes: 'Morning batch collection normal',
    user_id: 'usr-1',
    created_at: '2026-10-22T08:00:00Z',
  },
  {
    id: 'prod-rec-002',
    date: '2026-10-21',
    shed_name: 'Batch B',
    total_trays: 40.0,
    egg_count_good: 1200,
    egg_count_damaged: 1.0,
    dirty_cat1: 0.5,
    dirty_cat2: 0.0,
    production_percentage: 80.0,
    stock_in_trays: 80.0,
    mortality_count: 2,
    cull_count: 0,
    feed_consumed_kg: 0.0,
    flock_size: 1500,
    notes: 'High laying rate batch',
    user_id: 'usr-1',
    created_at: '2026-10-21T08:00:00Z',
  },
];

/**
 * Generate a mock JWT with future expiry to satisfy is_jwt_expired()
 */
function createMockJwt(): string {
  const header = Buffer.from(JSON.stringify({ alg: 'none', typ: 'JWT' })).toString('base64url');
  const payload = Buffer.from(
    JSON.stringify({
      sub: 'usr-mock-123',
      email: 'mockuser@farmiz.com',
      exp: Math.floor(Date.now() / 1000) + 86400 * 30, // 30 days in future
    })
  ).toString('base64url');
  return `${header}.${payload}.mockSignature`;
}

/**
 * Configure routes for authenticated mock session and Production endpoints
 */
async function setupMockRoutes(
  page: Page,
  records: Array<typeof mockProductionRecords[0]> = [...mockProductionRecords]
) {
  // Mock Sign-In endpoint
  await page.route(/\/api\/auth\/signin/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        token: createMockJwt(),
        session_cookies: ['session=mock-cookie-value'],
      }),
    });
  });

  // Mock Purchases endpoint (default landing route)
  await page.route(/\/api\/purchases/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify([]),
    });
  });

  // Mock Production list, create, update, delete
  await page.route(/\/api\/production(\/.*)?/, async (route) => {
    const method = route.request().method();
    const url = route.request().url();

    if (method === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(records),
      });
    } else if (method === 'POST') {
      const data = route.request().postDataJSON();
      const newRec = {
        id: `prod-rec-${Date.now()}`,
        ...data,
      };
      records.unshift(newRec);
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(newRec),
      });
    } else if (method === 'PUT') {
      const data = route.request().postDataJSON();
      const idMatch = url.match(/\/api\/production\/(.+)$/);
      const id = idMatch ? idMatch[1] : data.id;
      const idx = records.findIndex((r) => r.id === id);
      if (idx !== -1) {
        records[idx] = { ...records[idx], ...data };
      }
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(data),
      });
    } else if (method === 'DELETE') {
      const idMatch = url.match(/\/api\/production\/(.+)$/);
      if (idMatch) {
        const id = idMatch[1];
        const idx = records.findIndex((r) => r.id === id);
        if (idx !== -1) {
          records.splice(idx, 1);
        }
      }
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Deleted successfully' }),
      });
    } else {
      await route.continue();
    }
  });
}

/**
 * Sign in and navigate to the Production Log screen
 */
async function signInAndNavigateToProduction(page: Page) {
  await page.goto('/farmiz/signin');
  await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 15000 });

  await page.locator('#signin_email').fill('testuser@farmiz.com');
  await page.locator('#signin_password').fill('TestPassword123!');
  await page.getByRole('button', { name: 'Sign In' }).click();

  // Wait for sidebar navigation to appear
  await expect(page.getByText('Navigation', { exact: true })).toBeVisible({ timeout: 15000 });

  // Navigate to Production Log screen via sidebar menu button
  const productionNavBtn = page.getByRole('button', { name: /Production Log/i });
  await expect(productionNavBtn).toBeVisible({ timeout: 10000 });
  await productionNavBtn.click();

  // Verify Production Log header is displayed
  await expect(page.getByRole('heading', { name: /Production Log/i })).toBeVisible({ timeout: 15000 });
}

test.describe('Production Log Screen E2E Tests', () => {
  test.beforeEach(async ({ page }) => {
    // Clear storage before each test for clean session state
    await page.goto('/farmiz/');
    await page.evaluate(() => localStorage.clear());
  });

  test('should display Production Log screen header, filters, and action buttons', async ({ page }) => {
    await setupMockRoutes(page);
    await signInAndNavigateToProduction(page);

    // Verify Screen Title and Subtitle
    await expect(page.getByRole('heading', { name: /Production Log/i })).toBeVisible();
    await expect(
      page.getByText('Daily Batch Production, Dirty/Broken Egg Register & Stock Inventory')
    ).toBeVisible();

    // Verify Header Action Buttons
    await expect(page.getByRole('button', { name: /Manage Batches/i })).toBeVisible();
    await expect(page.getByRole('button', { name: /\+ Log Production|Log Production/i })).toBeVisible();

    // Verify Filter Controls
    await expect(page.getByPlaceholder('Search...')).toBeVisible();
    await expect(page.locator('select').filter({ hasText: 'All Batches' })).toBeVisible();
    await expect(page.locator('button[title="Table Layout"]')).toBeVisible();
    await expect(page.locator('button[title="Cards Layout"]')).toBeVisible();

    // Verify Date Range Filter Presets
    await expect(page.getByRole('button', { name: 'All Time' })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Today', exact: true })).toBeVisible();
    await expect(page.getByRole('button', { name: 'Last 7 Days', exact: true })).toBeVisible();
  });

  test('should render Production records in Table view with correct columns and summary totals', async ({ page }) => {
    await setupMockRoutes(page);
    await signInAndNavigateToProduction(page);

    // Verify Table Headers
    const table = page.locator('table');
    await expect(table).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Date' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Batch / Shed' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Total Trays' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Total Eggs' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Broken Egg Trays' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: /Dirty/i })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Yield %' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Stock (Trays)' })).toBeVisible();
    await expect(table.getByRole('columnheader', { name: 'Actions' })).toBeVisible();

    // Verify Row 1 content (Batch A)
    await expect(table.getByRole('cell', { name: '2026-10-22' })).toBeVisible();
    await expect(table.getByRole('cell', { name: 'Batch A' })).toBeVisible();
    await expect(table.getByRole('cell', { name: '1500' })).toBeVisible();

    // Verify Row 2 content (Batch B)
    await expect(table.getByRole('cell', { name: '2026-10-21' })).toBeVisible();
    await expect(table.getByRole('cell', { name: 'Batch B' })).toBeVisible();
    await expect(table.getByRole('cell', { name: '1200' })).toBeVisible();

    // Verify Summary Footer (tfoot)
    const tfoot = table.locator('tfoot');
    await expect(tfoot).toBeVisible();
    await expect(tfoot.getByRole('cell', { name: 'TOTAL' })).toBeVisible();
    await expect(tfoot.getByRole('cell', { name: '2 records' })).toBeVisible();
    await expect(tfoot.getByRole('cell', { name: '2700' })).toBeVisible(); // 1500 + 1200
  });

  test('should switch between Table Layout and Cards Layout', async ({ page }) => {
    await setupMockRoutes(page);
    await signInAndNavigateToProduction(page);

    // Initially in Table Layout
    await expect(page.locator('table')).toBeVisible();

    // Switch to Cards Layout
    await page.locator('button[title="Cards Layout"]').click();
    await expect(page.locator('table')).not.toBeVisible();

    // Verify Cards are displayed with batch badges and metrics
    await expect(page.getByText('🏷️ Batch A')).toBeVisible();
    await expect(page.getByText('🏷️ Batch B')).toBeVisible();
    await expect(page.getByText('Yield: 75.0%')).toBeVisible();
    await expect(page.getByText('Yield: 80.0%')).toBeVisible();

    // Switch back to Table Layout
    await page.locator('button[title="Table Layout"]').click();
    await expect(page.locator('table')).toBeVisible();
  });

  test('should filter records by Search Query and Batch Dropdown', async ({ page }) => {
    await setupMockRoutes(page);
    await signInAndNavigateToProduction(page);

    const searchInput = page.getByPlaceholder('Search...');

    // Search for "Batch B"
    await searchInput.fill('Batch B');
    await expect(page.locator('table').getByRole('cell', { name: 'Batch B' })).toBeVisible();
    await expect(page.locator('table').getByRole('cell', { name: 'Batch A' })).not.toBeVisible();
    await expect(page.locator('tfoot').getByRole('cell', { name: '1 records' })).toBeVisible();

    // Clear search
    await searchInput.fill('');
    await expect(page.locator('table').getByRole('cell', { name: 'Batch A' })).toBeVisible();
    await expect(page.locator('table').getByRole('cell', { name: 'Batch B' })).toBeVisible();

    // Filter using Batch dropdown
    const batchSelect = page.locator('select').filter({ hasText: 'All Batches' });
    await batchSelect.selectOption('Batch A');
    await expect(page.locator('table').getByRole('cell', { name: 'Batch A' })).toBeVisible();
    await expect(page.locator('table').getByRole('cell', { name: 'Batch B' })).not.toBeVisible();

    // Reset filter to All Batches
    await batchSelect.selectOption({ label: 'All Batches' });
    await expect(page.locator('table').getByRole('cell', { name: 'Batch B' })).toBeVisible();

    // Search for non-matching term to verify Empty State
    await searchInput.fill('NonExistentFlock999');
    await expect(page.getByText('No production records found')).toBeVisible();
    await expect(page.getByText('No production logs match your search criteria or batch/date filter.')).toBeVisible();
  });

  test('should validate Log Production form fields and 2-way tray/egg calculation', async ({ page }) => {
    await setupMockRoutes(page);
    await signInAndNavigateToProduction(page);

    // Open "+ Log Production" Sheet
    await page.getByRole('button', { name: /\+ Log Production|Log Production/i }).click();

    const dialog = page.getByRole('dialog');
    await expect(dialog.getByRole('heading', { name: 'Log Daily Production' })).toBeVisible();

    // Form inputs inside dialog
    const traysInput = dialog.locator('div:has(> label:has-text("Total Trays")) > input');
    const eggsInput = dialog.locator('div:has(> label:has-text("Total Eggs")) > input');
    const yieldInput = dialog.locator('input[step="0.1"]').nth(1);
    const brokenInput = dialog.locator('.grid-cols-3 > div').nth(0).locator('input');
    const dirty1Input = dialog.locator('.grid-cols-3 > div').nth(1).locator('input');
    const mortalityInput = dialog.locator('div:has(> label:has-text("Mortality")) > input');

    await expect(traysInput).toBeVisible();
    await expect(eggsInput).toBeVisible();
    await expect(yieldInput).toBeVisible();
    await expect(brokenInput).toBeVisible();
    await expect(dirty1Input).toBeVisible();
    await expect(mortalityInput).toBeVisible();

    // Test 2-way calculation: Trays -> Eggs (1 tray = 30 eggs)
    await traysInput.fill('10');
    await expect(eggsInput).toHaveValue('300');

    // Test 2-way calculation: Eggs -> Trays
    await eggsInput.fill('600');
    await expect(traysInput).toHaveValue('20');

    // Close sheet via Cancel button
    await dialog.getByRole('button', { name: 'Cancel' }).click();
    await expect(dialog).not.toBeVisible();
  });

  test('should create a new production log entry successfully', async ({ page }) => {
    const liveRecords = [...mockProductionRecords];
    await setupMockRoutes(page, liveRecords);
    await signInAndNavigateToProduction(page);

    // Open "+ Log Production" Sheet
    await page.getByRole('button', { name: /\+ Log Production|Log Production/i }).click();
    const dialog = page.getByRole('dialog');
    await expect(dialog.getByRole('heading', { name: 'Log Daily Production' })).toBeVisible();

    // Select Batch A inside dialog
    const shedSelect = dialog.getByRole('combobox');
    await shedSelect.selectOption('Batch A');

    // Fill form fields
    const traysInput = dialog.locator('div:has(> label:has-text("Total Trays")) > input');
    await traysInput.fill('35');

    const brokenInput = dialog.locator('.grid-cols-3 > div').nth(0).locator('input');
    await brokenInput.fill('1.5');

    const dirty1Input = dialog.locator('.grid-cols-3 > div').nth(1).locator('input');
    await dirty1Input.fill('0.5');

    const mortalityInput = dialog.locator('div:has(> label:has-text("Mortality")) > input');
    await mortalityInput.fill('3');

    const notesInput = dialog.getByPlaceholder('Vaccination, temperature, adjustments...');
    await notesInput.fill('End of day egg count logged by automation');

    // Submit form
    const saveBtn = dialog.getByRole('button', { name: 'Save Record' });
    await saveBtn.click();

    // Verify sheet closes and success toast is displayed
    await expect(dialog).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Production record logged successfully.')).toBeVisible({ timeout: 10000 });
  });

  test('should edit an existing production record', async ({ page }) => {
    const liveRecords = [...mockProductionRecords];
    await setupMockRoutes(page, liveRecords);
    await signInAndNavigateToProduction(page);

    // Click "Edit" on the first row (Batch A)
    const firstRowEditBtn = page.locator('table tbody tr').first().getByRole('button', { name: 'Edit' });
    await firstRowEditBtn.click();

    // Verify Edit sheet title inside dialog
    const dialog = page.getByRole('dialog');
    await expect(dialog.getByRole('heading', { name: 'Edit Production Record' })).toBeVisible();

    // Verify pre-populated values
    const traysInput = dialog.locator('div:has(> label:has-text("Total Trays")) > input');
    await expect(traysInput).toHaveValue('50');

    // Modify total trays
    await traysInput.fill('55');

    // Submit Update
    const updateBtn = dialog.getByRole('button', { name: 'Update Record' });
    await updateBtn.click();

    // Verify sheet closes and update success toast is displayed
    await expect(dialog).not.toBeVisible({ timeout: 10000 });
    await expect(page.getByText('Production record updated successfully.')).toBeVisible({ timeout: 10000 });
  });

  test('should open Delete confirmation dialog and cancel / confirm delete', async ({ page }) => {
    const liveRecords = [...mockProductionRecords];
    await setupMockRoutes(page, liveRecords);
    await signInAndNavigateToProduction(page);

    // Click "Delete" on the first row
    const firstRowDeleteBtn = page.locator('table tbody tr').first().getByRole('button', { name: 'Delete' });
    await firstRowDeleteBtn.click();

    // Verify Confirm Dialog appears
    await expect(page.getByRole('heading', { name: 'Delete Production Record' })).toBeVisible();
    await expect(
      page.getByText('Are you sure you want to delete this production record? This action cannot be undone.')
    ).toBeVisible();

    // Click Cancel first
    const dialogCancelBtn = page.locator('div[class*="fixed inset-0"]').getByRole('button', { name: 'Cancel' });
    await dialogCancelBtn.click();
    await expect(page.getByRole('heading', { name: 'Delete Production Record' })).not.toBeVisible();

    // Re-open Delete Dialog and Confirm
    await firstRowDeleteBtn.click();
    await expect(page.getByRole('heading', { name: 'Delete Production Record' })).toBeVisible();

    const dialogDeleteBtn = page.locator('div[class*="fixed inset-0"]').getByRole('button', { name: 'Delete' });
    await dialogDeleteBtn.click();

    // Verify success toast notification
    await expect(page.getByText('Production record deleted successfully.')).toBeVisible({ timeout: 10000 });
  });

  test('should manage farm batches in Manage Batches modal', async ({ page }) => {
    await setupMockRoutes(page);
    await signInAndNavigateToProduction(page);

    // Click "⚙️ Manage Batches" button
    await page.getByRole('button', { name: /Manage Batches/i }).click();

    // Verify Manage Batches modal
    const manageModal = page.locator('div:has(> div > h3:has-text("Manage Farm Batches"))');
    await expect(manageModal.getByRole('heading', { name: /Manage Farm Batches/i })).toBeVisible();

    // Add new batch: "Batch X"
    const batchInput = manageModal.getByPlaceholder('Enter batch name (e.g. Batch G, Shed 2)');
    await expect(batchInput).toBeVisible();
    await batchInput.fill('Batch X');

    await manageModal.getByRole('button', { name: '+ Add' }).click();

    // Verify Batch X is in the active list inside modal
    await expect(manageModal.locator('span', { hasText: 'Batch X' })).toBeVisible();

    // Close modal
    await manageModal.getByRole('button', { name: 'Done' }).click();
    await expect(manageModal).not.toBeVisible();

    // Open "+ Log Production" Sheet and verify Batch X is in dropdown
    await page.getByRole('button', { name: /\+ Log Production|Log Production/i }).click();
    const dialog = page.getByRole('dialog');
    const shedSelect = dialog.getByRole('combobox');
    await expect(shedSelect.locator('option', { hasText: 'Batch X' })).toBeAttached();
  });

  /**
   * Real API Flow: Production Log screen with live Neon backend API
   */
  test('should load Production Log screen using real user credentials (Real API)', async ({ page }) => {
    test.skip(
      !hasRealCredentials,
      'Skipped: Set E2E_EMAIL and E2E_PASSWORD secrets in GitHub Actions or .env to run Real API production log test'
    );

    await page.goto('/farmiz/signin');
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 15000 });

    // Fill real credentials
    await page.locator('#signin_email').fill(e2eEmail!);
    await page.locator('#signin_password').fill(e2ePassword!);

    // Submit Sign In
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Verify transition to authenticated navigation
    await expect(page.getByText('Navigation', { exact: true })).toBeVisible({ timeout: 15000 });

    // Navigate to Production Log screen
    const productionNavBtn = page.getByRole('button', { name: /Production Log/i });
    await expect(productionNavBtn).toBeVisible({ timeout: 10000 });
    await productionNavBtn.click();

    // Verify Production Log header
    await expect(page.getByRole('heading', { name: /Production Log/i })).toBeVisible({ timeout: 15000 });

    // Verify either table of records or empty state renders successfully
    const contentVisible = await Promise.race([
      page.locator('table').waitFor({ timeout: 10000 }).then(() => true).catch(() => false),
      page.getByText('No production records found').waitFor({ timeout: 10000 }).then(() => true).catch(() => false),
    ]);
    expect(contentVisible).toBe(true);
  });
});
