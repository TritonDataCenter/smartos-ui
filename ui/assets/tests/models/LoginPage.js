const { expect } = require('@playwright/test')
const Page = require('./Page')
const { defaultUser, defaultPassword } = require('../config')

class LoginPage extends Page {
  constructor (page) {
    super(page, '/', 'SmartOS UI Login')
    this.userInput = page.getByRole('textbox', { name: 'user' })
    this.passwordInput = page.getByRole('textbox', { name: 'password' })
    this.signInButton = page.getByRole('button', { hasText: 'Sign in' })
  }

  async loginSuccess () {
    await this.userInput.fill(defaultUser)
    await this.passwordInput.fill(defaultPassword)
    await this.signInButton.click()
    await expect(this.page).toHaveTitle('Dashboard')
  }

  async loginFailure () {
    await this.userInput.fill(defaultUser)
    await this.passwordInput.fill(defaultPassword + 'blahblah')
    await this.signInButton.click()
    await expect(this.page.getByText('Invalid username or password')).toBeVisible()
  }

  async login () {
    await this.goto()
    await this.loginSuccess()
  }
}

module.exports = LoginPage
