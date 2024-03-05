const { expect } = require('@playwright/test')
const { URL } = require('../config')

class Page {
  validateTitle = true
  constructor (page, path, title) {
    this.page = page
    this.path = path
    this.title = title
  }

  async goto () {
    await this.page.goto(`${URL}${this.path}`)
    if (this.validateTitle) {
      await expect(this.page).toHaveTitle(this.title, { timeout: 10000 })
    }
  }

  async firstLoad () {
    await expect(this.page.getByText('No images are installed')).toBeVisible()
    await expect(this.page.getByRole('button', { hasText: 'Import an Image' })).toBeVisible()
  }

  async firstLoadNoInstances () {
    await expect(this.page.getByText('No instances have been created')).toBeVisible()
    await expect(this.page.getByRole('button', { hasText: 'Create an instance' })).toBeVisible()
  }

  getNotification (uuid) {
    return [
      this.page.locator(`.notification[data-for-entity="${this.uuid}"]`),
      this.page.locator(`.notification[data-for-entity="${this.uuid}"] .notification-close`)
    ]
  }

  async changeSelect (selector, value) {
    const locator = this.page.locator(selector)
    await locator.selectOption({ value })
    await locator.blur()
    await locator.dispatchEvent('change')
  }
}

module.exports = Page