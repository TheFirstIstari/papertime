import { test, expect } from '@playwright/test';

test.describe('PaperTime — smoke tests', () => {

	test('homepage loads and shows search form after hydration', async ({ page }) => {
		await page.goto('/');
		const input = page.locator('input[placeholder*="Origin"]');
		await expect(input).toBeVisible({ timeout: 15000 });
	});

	test('search by station name shows results', async ({ page }) => {
		await page.goto('/');
		// Wait for hydration
		const originInput = page.locator('input[placeholder*="Origin"]');
		await expect(originInput).toBeVisible({ timeout: 15000 });

		// Type origin — this triggers the autocomplete suggestions
		await originInput.fill('London');
		await page.waitForTimeout(500);

		// The suggestion dropdown appears; click London Euston
		const suggestion = page.getByRole('button').filter({ hasText: 'Euston' });
		await expect(suggestion).toBeVisible({ timeout: 3000 });
		await suggestion.click();
		await page.waitForTimeout(200);

		// Fill destination
		const destInput = page.locator('input[placeholder*="Destination"]');
		await destInput.fill('Birmingham');
		await page.waitForTimeout(500);

		// Click the suggestion for Birmingham New Street
		const destSuggestion = page.getByRole('button').filter({ hasText: 'New Street' });
		await expect(destSuggestion).toBeVisible({ timeout: 3000 });
		await destSuggestion.click();
		await page.waitForTimeout(200);

		// Click Search
		await page.locator('button[type="submit"]').click();

		// Should show results
		await expect(page.locator('a[href*="/table/"]').first()).toBeVisible({ timeout: 10000 });
	});

	test('table detail page loads with formatted times', async ({ page }) => {
		await page.goto('/table/001?from=FST&to=SRA');
		await expect(page.locator('table')).toBeVisible({ timeout: 15000 });
		const tableText = await page.locator('table').textContent() ?? '';
		expect(tableText).toMatch(/\d{2}:\d{2}/);
	});

	test('marey index page loads and lists tables', async ({ page }) => {
		await page.goto('/marey');
		await expect(page.locator('text=Marey')).toBeVisible({ timeout: 10000 });
	});

	test('marey detail page renders chart', async ({ page }) => {
		await page.goto('/marey/t001');
		await expect(page.locator('svg').first()).toBeVisible({ timeout: 15000 });
	});

	test('table detail page highlights from/to stations', async ({ page }) => {
		await page.goto('/table/001?from=EUS&to=BHM');
		await expect(page.locator('table')).toBeVisible({ timeout: 15000 });
	});

	test('popular routes show station names not CRS codes', async ({ page }) => {
		await page.goto('/');
		// Wait for hydration
		const btn = page.locator('button:has-text("Birmingham")');
		await expect(btn).toBeVisible({ timeout: 15000 });
	});

	test('404 page shows for unknown routes', async ({ page }) => {
		await page.goto('/nonexistent-page');
		await expect(page.locator('text=404')).toBeVisible({ timeout: 10000 });
	});
});
