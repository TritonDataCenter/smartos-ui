const { expect } = require('@playwright/test')
const { imageImportTimeout } = require('../config')
const Page = require('./Page')

class AvailableImagesPage extends Page {
  constructor (page) {
    super(page, '/import', 'Available Images')
  }

  async import (name, uuid) {
    const title = `Import ${name} (${uuid})`
    await this.page.getByTitle(title).click()
    const successText = `Image ${uuid} has been imported and is ready to use.`
    const locator = this.page.locator(`#notification-${uuid} .notification-body`)
    await expect(locator).toContainText(successText, imageImportTimeout)
  }

  async isNotAvailable (name, uuid) {
    const title = `Import ${name} (${uuid})`
    await expect(this.page.getByTitle(title)).not.toBeVisible()
  }
}

module.exports = AvailableImagesPage
