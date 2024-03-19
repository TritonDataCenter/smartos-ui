/*
 *
 * This test assumes first-run.spec.js has already been executed successfully
 * or the required images have already been imported.
 *
 * This test will confirm that images can be deleted and viewed. It import and
 * delete the image specified in the config as deleteTestImage if you have
 * instances that depend on this image, this test will fail (feel free to
 * change it to any other image.)
 *
 */
const { test } = require('@playwright/test')
const { images, imageImportTimeout, deleteTestImage } = require('./config')
const LoginPage = require('./models/LoginPage')
const ImagesPage = require('./models/ImagesPage')
const ImagePage = require('./models/ImagePage')
const AvailableImagesPage = require('./models/AvailableImagesPage')

test.use({
  ignoreHTTPSErrors: true
})

test('Check installed images', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const imagesPage = new ImagesPage(page)
  await imagesPage.goto()

  // Confirm all images exist in the images list
  for (const i in images) {
    await imagesPage.isImported(images[i].uuid)
  }

  const availPage = new AvailableImagesPage(page)
  await availPage.goto()

  // Confirm they _don't_ exist in the available images list
  for (const i in images) {
    await availPage.isNotAvailable(images[i].name, images[i].uuid)
  }

  // Confirm we can load the page for each one and toggle json view
  for (const i in images) {
    await imagesPage.goToImage(images[i].uuid)
    const imagePage = new ImagePage(page, images[i].name, images[i].uuid)
    await imagePage.JSONViewToggle()
  }
})

test('Delete Sacrificial Image', async ({ page }) => {
  test.setTimeout(imageImportTimeout.timeout)
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const availPage = new AvailableImagesPage(page)

  const imagesPage = new ImagesPage(page)
  await imagesPage.goto()

  const row = await imagesPage.getImageRow(deleteTestImage.uuid)
  const imported = Boolean(await row.count())

  if (!imported) {
    await availPage.goto()
    await availPage.import(deleteTestImage.name, deleteTestImage.uuid)
  }

  await imagesPage.goToImage(deleteTestImage.uuid)

  const imagePage = new ImagePage(page, deleteTestImage.name, deleteTestImage.uuid)
  await imagePage.goto()
  await imagePage.deleteImage()
})
