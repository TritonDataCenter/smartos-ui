const Page = require('./Page')
const InstanceNativePage = require('./InstanceNativePage')

class InstanceJoyentPage extends InstanceNativePage {
  brand = 'joyent'
  constructor (page, uuid) {
    super(page, uuid)
  }
}

module.exports = InstanceJoyentPage
