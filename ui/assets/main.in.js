import htmx from 'htmx.org'
import { $, $$ } from './global'
import { setupProvisioningForm } from './provision'
import { setupJSONViewer } from './json-viewer'
import { removeMe } from './htmx-extensions'

import 'sortable-tablesort'

removeMe(htmx)

document.addEventListener('DOMContentLoaded', () => {

  // Handle closing modals
  $('#modal').addEventListener('click', e => {
    if (e.target.classList.contains('modal-close') ||
      e.target.closest('.modal-close')) {
      $('#modal-content').classList.add('hidden')
    }
  })

  // Handle closing notifications
  $('#notifications').addEventListener('click', e => {
    const $target = e.target.classList.contains('notification-close')
      ? e.target
      : e.target.closest('.notification-close')
    if ($target) {
      const $notification = $($target.dataset.target)
      if ($notification) {
        $notification.remove()
      }
    }
  })

  // Update the sidebar navigation to show the currently selected view.
  // This will occur automatically on a full page reload but we must handle it
  // on our own when navigating via HTMX.
  htmx.on('htmx:pushedIntoHistory', ({ detail: { path } }) => {
    const active = 'active-sidebar-nav'
    const inactive = 'inactive-sidebar-nav'
    $$('.main-nav').forEach($element => {
      const match = $element.dataset.hxGet === path
      $element.classList.remove(match ? inactive : active)
      $element.classList.add(match ? active : inactive)
    })
  })

  // The provisioning form has a bit of extra javascript to setup the JSON
  // editors, merge additional properties and build NIC objects
  // this has to be initialized dynamically as the elements we rely on come
  // and go in the DOM
  htmx.on('htmx:afterSettle', ({ detail: { pathInfo: { requestPath } } }) => {
    if (requestPath === '/provision') {
      setupProvisioningForm()
    } else {
      setupJSONViewer()
    }
  })

  // If user does a full refresh or navigates to the provision page manually
  // we need to setup the forms here too.
  if (document.location.pathname === '/provision') {
    setupProvisioningForm()
  } else {
    setupJSONViewer()
  }

  // When a notification is loaded which contains a redirect, it will also
  // have a data-created-at attribute. If the current path does not equal
  // the value in that attribute, do not redirect.
  htmx.on('htmx:beforeRequest', (e) => {
    if (e.target.dataset.createdAt &&
      document.location.pathname !== e.target.dataset.createdAt) {
      e.preventDefault()
    }
  })
})
