import htmx from 'htmx.org'
import { $, $$ } from './global'
import { setupProvisioningForm } from './provision'

document.addEventListener('DOMContentLoaded', () => {
  const $notify = $('#notify')

  function hideNotify () {
    $notify.classList.add('hidden')
  }

  function showNotify () {
    $notify.classList.remove('hidden')
  }

  $('#notify-close').addEventListener('click', hideNotify)

  function notify (notification, timeout = 10000) {
    if (!notification || !notification.heading || !notification.body) {
      return
    }

    const $heading = $('#notify-body')
    const $body = $('#notify-heading')
    const icons = {
      ok: $('#notifiy-icon-ok'),
      error: $('#notifiy-icon-error'),
      info: $('#notifiy-icon-info')
    }
    const icon = notification.icon || 'ok'

    $heading.innerText = notification.heading
    $body.innerText = notification.body

    icons[icon].classList.remove('hidden')

    showNotify()

    setTimeout(() => {
      hideNotify()
      icons.ok.classList.add('hidden')
      icons.error.classList.add('hidden')
      icons.info.classList.add('hidden')
    }, timeout)
  }

  // Update the sidebar to show the currently selected view.
  // This will occur automatically on a full page reload but we must handle it
  // on our own when navigating via HTMX.
  htmx.on('htmx:pushedIntoHistory', ({ detail: { path } }) => {
    const active = 'active-sidebar-nav'
    const inactive = 'inactive-sidebar-nav'
    $$('.main-nav').forEach($element => {
      const match = $element.getAttribute('hx-get') === path
      $element.classList.remove(match ? inactive : active)
      $element.classList.add(match ? active : inactive)
    })
  })

  // Long running requests such as importing images or creating an instance
  // may send their response when the user has navigated to another view within
  // the app which will then redirect them to the path in the HX-Location header
  // which may be undesirable. To account for this, these long running requests
  // send back a list of paths that are acceptable for redirecting. If the user
  // is not currently at one of these paths a notification will be shown.
  htmx.on('htmx:beforeRequest', event => {
    const { detail: { etc: { values } } } = event
    if (values) {
      const { allowedPaths, notification } = values
      const { pathname } = document.location

      // Don't redirect, just show notification
      if (allowedPaths.indexOf(pathname) === -1) {
        event.preventDefault()
      }

      notify(notification)
    }
  })

  // The provisioning form as a bit of extra javascript to setup the JSON
  // editors, merge additional properties and build NIC objects
  // this has to be initialized dynamically as the elements we rely on come
  // and go in the DOM
  htmx.on('htmx:afterSettle', ({ detail: { pathInfo: { requestPath } } }) => {
    if (requestPath === '/provision') {
      setupProvisioningForm()
    }
  })

  // if user does a full refresh or navigates to the provision page manually
  // need to setup the forms here too.
  if (document.location.pathname === '/provision') {
    setupProvisioningForm()
  }
})
