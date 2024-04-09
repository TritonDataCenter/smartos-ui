/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

import { $, $$ } from './global'

// Client-side table filtering
document.addEventListener('DOMContentLoaded', () => {
  const $main = $('#main')

  function filterTable (e) {
    const target = e.target

    // If ESC key is pressed, clear out input
    if (e.keyCode === 27) {
      target.value = ''
    }

    if (target.classList.contains('filterable')) {
      const value = target.value.trim().toLowerCase()
      const $rows = $$(target.dataset.rows)
      if (value.length === 0) {
        target.nextElementSibling.classList.add('hidden')
        $rows.forEach(tr => tr.classList.remove('hidden'))
        return
      }
      target.nextElementSibling.classList.remove('hidden')
      $rows.forEach(tr => {
        tr.classList.add('hidden')
        tr.querySelectorAll('.filter-subject').forEach(s => {
          if (s.textContent.toLowerCase().includes(value)) {
            tr.classList.remove('hidden')
          }
        })
      })
    }
  }

  $main.addEventListener('keyup', filterTable)
  $main.addEventListener('change', filterTable)
  $main.addEventListener('blur', filterTable)
  $main.addEventListener('click', ({ target }) => {
    const $target = target.classList.contains('filter-clear')
      ? target
      : target.closest('.filter-clear')
    if ($target) {
      const $input = $target.previousElementSibling
      $input.value = ''
      filterTable({ target: $input })
    }
  })

  /*
   * This is intended to be called via a HX-Trigger-After-Swap header and allows
   * for re-filtering a table if the content has been reloaded.
   */
  document.body.addEventListener('filterTable', ({ detail: { selector } }) => {
    filterTable({ target: $(selector) })
  })
})
