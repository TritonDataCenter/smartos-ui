const { expect } = require('@playwright/test')
const Page = require('./Page')

class ImagePage extends Page {
  constructor (page, name, uuid) {
    super(page, `/images/${uuid}`, `Image: ${name}`)
    this.name = name
    this.uuid = uuid
  }

  async deleteImage () {
    const button = await this.page.locator(`button[data-hx-delete="/images/${this.uuid}"]`)
    this.page.on('dialog', dialog => dialog.accept())
    await button.click()
    const successText = `Image ${this.name} (${this.uuid}) successfully deleted`
    const locator = this.page.locator(`#notification-${this.uuid} .notification-body`)
    await expect(locator).toContainText(successText)
  }

  async JSONViewToggle () {
    const jsonButton = await this.page.locator(`button[data-hx-get="/images/${this.uuid}?json=true"]`)
    await jsonButton.click()
    await expect(this.page.locator('.cm-editor')).toBeAttached()
    const propsButton = await this.page.locator(`button[data-hx-get="/images/${this.uuid}"]`)
    await propsButton.click()
    await expect(this.page.locator('.cm-editor')).not.toBeAttached()
  }
}

module.exports = ImagePage
