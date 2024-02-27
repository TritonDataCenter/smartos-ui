const Page = require('./Page')
const { expect } = require('@playwright/test')

class ProvisionPage extends Page {
  constructor (page) {
    super(page, '/provision', 'Create Instance')
  }

  async selectImage(uuid) {
    await this.page.locator('#image_uuid').selectOption({value: uuid})
  }

  async confirmBrand(brand) {
    await expect(await this.page.locator('#brand')).toHaveValue(brand)
  }

  async validate() {
    await this.page.locator('#validate-button').click()
    const validateResult = this.page.locator('#validate-result')
    await expect(validateResult).toContainText('Valid Instance Details')
  }

  async create() {
    await this.page.locator('#create-button').click()
    const createResult = this.page.locator('#modal-title')
  }

  async createSuccess() {
    const locator = this.page.locator('#modal-title')
    await expect(locator).toContainText('Provision successful')
  }

  async instanceList() {

  }

  async instanceDetails() {

  }
}

module.exports = ProvisionPage