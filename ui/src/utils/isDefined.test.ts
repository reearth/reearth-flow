import { expect, test } from 'vitest'
import { isDefined } from './isDefined'

test('check if not defined', () => {
  let a: number | undefined = undefined
  expect(isDefined(a)).toBe(false)
})

test('check if defined', () => {
  let a: number | undefined = 1
  expect(isDefined(a)).toBe(true)
})