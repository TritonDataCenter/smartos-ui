const { expect } = require('@playwright/test')
const Page = require('./Page')

class ImagesPage extends Page {
  constructor (page) {
    super(page, '/images', 'Images')
    this.availableLink = this.page.locator('a[href="/import"]')
  }

  async availableImages () {
    await this.availableLink.click()
    await expect(this.page).toHaveTitle('Available Images')
  }

  async getImageRow (uuid) {
    return this.page.locator(`tbody tr[data-hx-get="/images/${uuid}"]`)
  }

  async isImported (uuid) {
    const imageRow = await this.getImageRow(uuid)
    await expect(imageRow).toBeVisible()
  }

  async goToImage (uuid) {
    await this.goto()
    const imageRow = await this.getImageRow(uuid)
    await imageRow.click()
  }
}

module.exports = ImagesPage
