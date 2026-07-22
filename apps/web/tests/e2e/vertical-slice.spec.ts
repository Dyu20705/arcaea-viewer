import { expect, test } from "@playwright/test";

test("renders the checkpoint fixture through WASM", async ({ page }) => {
  const pageErrors: Error[] = [];
  page.on("pageerror", (error) => pageErrors.push(error));

  await page.goto("/");

  await expect(page.getByText("WASM", { exact: true })).toBeVisible();
  await expect(page.getByText("ready")).toBeVisible({ timeout: 20_000 });

  await page.getByRole("button", { name: "Parse & Render" }).click();

  await expect(page.getByText("success")).toBeVisible();
  await expect(
    page.getByLabel("Render summary").getByText("Primitive count", { exact: true }),
  ).toBeVisible();
  await expect(page.locator('[data-testid="svg-viewer"] svg')).toBeVisible();

  const firstPlayback = await page
    .locator('[data-testid="svg-viewer"] svg')
    .textContent();
  await page.getByLabel("Playback time").fill("3200");
  await page.getByRole("button", { name: "Parse & Render" }).click();
  await expect(page.locator('[data-testid="svg-viewer"] svg')).toContainText(
    "Playback: 3200ms",
  );
  const secondPlayback = await page
    .locator('[data-testid="svg-viewer"] svg')
    .textContent();

  expect(firstPlayback).not.toEqual(secondPlayback);
  expect(pageErrors).toEqual([]);
});
