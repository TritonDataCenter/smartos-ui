const Page = require('./Page')
const { expect } = require('@playwright/test')

class InstancePage extends Page {
  brand = 'unknown'
  validateTitle = false
  constructor (page, uuid) {
    super(page, `/instances/${uuid}`)
    this.uuid = uuid
    this.page.on('dialog', dialog => dialog.accept())
  }

  async isExpectedBrand() {
    await expect(this.page.locator('#prop-brand')).toHaveValue(this.brand)
  }

  async stop() {
    const stopButton = await this.page.locator(`button[data-hx-post="/instances/${this.uuid}/stop"]`)
    await expect(stopButton).toContainText('Stop')
    stopButton.click()
    const successText = `Instance ${this.uuid} successfully stopped`
    const [locator, close] = this.getNotification(this.uuid)
    await expect(locator).toContainText(successText)
    await close.click()
  }

  async start() {
    const stopButton = await this.page.locator(`button[data-hx-post="/instances/${this.uuid}/start"]`)
    await expect(stopButton).toContainText('Start')
    stopButton.click()
    const successText = `Instance ${this.uuid} successfully started`
    const [locator, close] = this.getNotification(this.uuid)
    await expect(locator).toContainText(successText)
    await close.click()
  }

  async deleteInstance() {
    const deleteButton = await this.page.locator(`button[data-hx-delete="/instances/${this.uuid}"]`)
    await expect(deleteButton).toContainText('Delete')
    deleteButton.click()
    const successText = `(${this.uuid}) successfully deleted`
    const [locator, close] = this.getNotification(this.uuid)
    await expect(locator).toContainText(successText)
    await close.click()
    // Should navigate back to instances page
    await expect(this.page).toHaveTitle('Instances', {timeout: 10000})
  }

  async isRunning() {
    await expect(this.page.locator('#prop-state')).toHaveValue('running')
  }
}

module.exports = InstancePage
