import { expect, test } from 'vitest'
import { randomID } from './randomID'

test('random id with a default length of 10', () => {
  expect(randomID().length).toBe(10)
})

test('random id with a specified length of 4', () => {
  expect(randomID(4).length).toBe(4)
})

test('two ids are not equal', () => {
  expect(randomID()).not.toBe(randomID())
})