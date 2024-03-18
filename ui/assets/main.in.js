import htmx from 'htmx.org'
import { $, $$ } from './global'
import { setupProvisioningForm } from './provision'
import { setupJSONViewer } from './json-viewer'
import { removeMe } from './htmx-extensions'

import './tableFilter'

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
    if (requestPath === '/provision' &&
      document.location.pathname === '/provision') {
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
  htmx.on('htmx:beforeRequest', e => {
    if (e.target.dataset.createdAt &&
      document.location.pathname !== e.target.dataset.createdAt) {
      e.preventDefault()
    }
  })

  // If you click the "Available Images" button on the Images view, then quickly
  // navigate elsewhere before it has loaded, you will be redirected back to the
  // "Available Images" view when it loads instead of aborting the request like
  // the other side-panel navigation entries which use hx-sync. Using hx-sync
  // higher in the DOM tree (such as on the body element) breaks how we handle
  // notifications and modals so this event handler is used to prevent the
  // unexpected redirect from occurring.
  // We should probably implement a more generic convention for handling these
  // kind of situations but the "Available Images" button is currently the only
  // place this occurs.
  htmx.on('htmx:beforeSwap', e => {
    const { detail: { pathInfo: { requestPath } } } = e
    if (requestPath === '/import' && document.location.pathname !== '/images') {
      e.preventDefault()
    }
  })

  // Respond to an element removal request from the server. This is currently
  // used when an image is successfully imported. The row in the table of
  // available images is removed without a full page or view reload.
  document.body.addEventListener('removeElement', ({ detail: { id } }) => {
    const element = document.getElementById(id)
    if (!element) {
      console.error('Requested element to remove not found', id)
    }
    element.remove()
  })
})
