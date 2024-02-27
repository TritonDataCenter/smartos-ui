const { expect } = require('@playwright/test')
const { URL } = require('../config')

class Page {
  constructor (page, path, title) {
    this.page = page
    this.path = path
    this.title = title
  }

  async goto () {
    await this.page.goto(`${URL}${this.path}`)
    await expect(this.page).toHaveTitle(this.title, {timeout: 10000})
  }

  async firstLoad () {
    await expect(this.page.getByText('No images are installed')).toBeVisible()
    await expect(this.page.getByRole('button', { hasText: 'Import an Image' })).toBeVisible()
  }

  async firstLoadNoInstances () {
    await expect(this.page.getByText('No instances have been created')).toBeVisible()
    await expect(this.page.getByRole('button', { hasText: 'Create an instance' })).toBeVisible()
  }
}

module.exports = Page
