const { expect } = require('@playwright/test')
const Page = require('./Page')

class DashboardPage extends Page {
  constructor (page) {
    super(page, '/dashboard', 'Dashboard')
    this.dashboardLink = page.locator('#navigation a[href="/dashboard"]')
    this.instancesLink = page.locator('#navigation a[href="/instances"]')
    this.imagesLink = page.locator('#navigation a[href="/images"]')
    this.logoutLink = page.locator('#navigation a[href="/logout"]')
  }

  async goto () {
    await super.goto()
    await expect(this.dashboardLink).toBeVisible()
    await expect(this.instancesLink).toBeVisible()
    await expect(this.imagesLink).toBeVisible()
    await expect(this.logoutLink).toBeVisible()
  }

  async dashboard () {
    await this.dashboardLink.click()
    await expect(this.page).toHaveTitle('Dashboard')
  }

  async instances () {
    await this.instancesLink.click()
    await expect(this.page).toHaveTitle('Instances')
  }

  async images () {
    await this.imagesLink.click()
    await expect(this.page).toHaveTitle('Images')
  }

  async logout () {
    await this.logoutLink.click()
    await expect(this.page).toHaveTitle('SmartOS UI Login')
  }
}

module.exports = DashboardPage
