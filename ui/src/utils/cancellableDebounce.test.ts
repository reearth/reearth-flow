import { cancellableDebounce } from "./cancellableDebounce";

describe("cancellableDebounce", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  it("should debounce the function call", () => {
    const func = vi.fn();
    const wait = 100;
    const debouncedFunc = cancellableDebounce(func, wait);

    debouncedFunc();

    expect(func).not.toHaveBeenCalled();

    vi.advanceTimersByTime(wait);

    expect(func).toHaveBeenCalled();
  });

  it("should cancel the debounced function call", () => {
    const func = vi.fn();
    const wait = 100;
    const debouncedFunc = cancellableDebounce(func, wait);

    debouncedFunc();
    debouncedFunc.cancel();

    vi.advanceTimersByTime(wait);

    expect(func).not.toHaveBeenCalled();
  });

  it("should debounce successive calls and only call the last one", () => {
    const func = vi.fn();
    const wait = 100;
    const debouncedFunc = cancellableDebounce(func, wait);

    debouncedFunc();
    vi.advanceTimersByTime(wait / 2);

    debouncedFunc();
    vi.advanceTimersByTime(wait / 2);

    debouncedFunc();
    vi.advanceTimersByTime(wait / 2);

    debouncedFunc();
    vi.advanceTimersByTime(wait);

    expect(func).toHaveBeenCalledTimes(1);
  });

  it("should pass the correct arguments to the debounced function", () => {
    const func = vi.fn();
    const wait = 100;
    const debouncedFunc = cancellableDebounce(func, wait);

    debouncedFunc("arg1", "arg2");

    vi.advanceTimersByTime(wait);

    expect(func).toHaveBeenCalledWith("arg1", "arg2");
  });
});
