const Page = require('./Page')

class InstancesPage extends Page {
  constructor (page) {
    super(page, '/instances', 'Instances')
  }
}

module.exports = InstancesPage
