import { formatDate } from './date'

test('format date to be human readable', () => {
  expect(formatDate('2021-01-01T00:00:00.000Z')).toBe('2021-1-1 9:00')
})