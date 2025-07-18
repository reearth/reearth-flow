import { describe, it, expect } from 'vitest';

import { setValueAtPath, getValueAtPath, extractFieldPath, createFieldContext } from './fieldUtils';

describe('setValueAtPath', () => {
  describe('basic functionality', () => {
    it('should set value at shallow path', () => {
      const obj = { a: 1, b: 2 };
      const result = setValueAtPath(obj, ['a'], 10);
      
      expect(result).toEqual({ a: 10, b: 2 });
      expect(result).not.toBe(obj); // Should be immutable
    });

    it('should set value at deep path', () => {
      const obj = { a: { b: { c: 1 } } };
      const result = setValueAtPath(obj, ['a', 'b', 'c'], 10);
      
      expect(result).toEqual({ a: { b: { c: 10 } } });
      expect(result).not.toBe(obj); // Should be immutable
    });

    it('should create missing intermediate objects', () => {
      const obj = {};
      const result = setValueAtPath(obj, ['a', 'b', 'c'], 10);
      
      expect(result).toEqual({ a: { b: { c: 10 } } });
    });

    it('should return value directly when path is empty', () => {
      const obj = { a: 1 };
      const result = setValueAtPath(obj, [], 'new value');
      
      expect(result).toBe('new value');
    });
  });

  describe('array preservation', () => {
    it('should preserve array structure when setting values in arrays', () => {
      const obj = {
        operations: [
          { method: 'create', value: '12345', attribute: 'myAwesomeAttribute' }
        ]
      };
      
      const result = setValueAtPath(obj, ['operations', '0', 'value'], '67890');
      
      expect(result.operations).toBeInstanceOf(Array);
      expect(result.operations[0].value).toBe('67890');
      expect(result.operations[0].method).toBe('create');
      expect(result.operations[0].attribute).toBe('myAwesomeAttribute');
    });

    it('should handle nested arrays correctly', () => {
      const obj = {
        data: [
          { items: [1, 2, 3] },
          { items: [4, 5, 6] }
        ]
      };
      
      const result = setValueAtPath(obj, ['data', '0', 'items', '1'], 999);
      
      expect(result.data).toBeInstanceOf(Array);
      expect(result.data[0].items).toBeInstanceOf(Array);
      expect(result.data[0].items[1]).toBe(999);
    });

    it('should handle arrays as root objects', () => {
      const obj = [
        { name: 'item1', value: 10 },
        { name: 'item2', value: 20 }
      ];
      
      const result = setValueAtPath(obj, ['0', 'value'], 100);
      
      expect(Array.isArray(result)).toBe(true);
      expect(result[0].value).toBe(100);
      expect(result[1].value).toBe(20);
    });

    it('should preserve empty arrays', () => {
      const obj = { data: [] };
      const result = setValueAtPath(obj, ['data', '0'], 'new item');
      
      expect(result.data).toBeInstanceOf(Array);
      expect(result.data[0]).toBe('new item');
    });
  });

  describe('mixed structures', () => {
    it('should handle objects within arrays', () => {
      const obj = {
        items: [
          { config: { enabled: true } },
          { config: { enabled: false } }
        ]
      };
      
      const result = setValueAtPath(obj, ['items', '1', 'config', 'enabled'], true);
      
      expect(result.items).toBeInstanceOf(Array);
      expect(result.items[1].config.enabled).toBe(true);
    });

    it('should handle arrays within objects', () => {
      const obj = {
        settings: {
          permissions: ['read', 'write'],
          metadata: { version: 1 }
        }
      };
      
      const result = setValueAtPath(obj, ['settings', 'permissions', '1'], 'admin');
      
      expect(result.settings.permissions).toBeInstanceOf(Array);
      expect(result.settings.permissions[1]).toBe('admin');
    });
  });

  describe('edge cases', () => {
    it('should handle null values', () => {
      const obj = { a: null };
      const result = setValueAtPath(obj, ['a', 'b'], 'value');
      
      expect(result.a.b).toBe('value');
    });

    it('should handle undefined values', () => {
      const obj = { a: undefined };
      const result = setValueAtPath(obj, ['a', 'b'], 'value');
      
      expect(result.a.b).toBe('value');
    });

    it('should handle primitive values being overwritten', () => {
      const obj = { a: 'string' };
      const result = setValueAtPath(obj, ['a', 'b'], 'value');
      
      expect(result.a.b).toBe('value');
    });

    it('should handle numeric array indices', () => {
      const obj = { items: ['a', 'b', 'c'] };
      const result = setValueAtPath(obj, ['items', '10'], 'new item');
      
      expect(result.items).toBeInstanceOf(Array);
      expect(result.items[10]).toBe('new item');
    });
  });

  describe('user-reported bug case', () => {
    it('should fix the specific array-to-object conversion issue', () => {
      const originalValue = {
        operations: [
          {
            method: 'create',
            value: '12345',
            attribute: 'myAwesomeAttribute2'
          }
        ]
      };
      
      // This should update the value while preserving the array structure
      const result = setValueAtPath(originalValue, ['operations', '0', 'value'], '67890');
      
      // The critical test: operations should still be an array
      expect(Array.isArray(result.operations)).toBe(true);
      expect(result.operations.length).toBe(1);
      expect(result.operations[0].value).toBe('67890');
      expect(result.operations[0].method).toBe('create');
      expect(result.operations[0].attribute).toBe('myAwesomeAttribute2');
      
      // Should not have been converted to object (operations.0 would be different from operations[0])
      expect('0' in result.operations).toBe(true); // Arrays have numeric indices
      expect(result.operations.constructor).toBe(Array); // But they're still arrays
    });
  });
});

describe('getValueAtPath', () => {
  it('should get value at shallow path', () => {
    const obj = { a: 1, b: 2 };
    const result = getValueAtPath(obj, ['a']);
    
    expect(result).toBe(1);
  });

  it('should get value at deep path', () => {
    const obj = { a: { b: { c: 1 } } };
    const result = getValueAtPath(obj, ['a', 'b', 'c']);
    
    expect(result).toBe(1);
  });

  it('should return undefined for non-existent paths', () => {
    const obj = { a: 1 };
    const result = getValueAtPath(obj, ['b', 'c']);
    
    expect(result).toBeUndefined();
  });

  it('should get values from arrays', () => {
    const obj = { items: [1, 2, 3] };
    const result = getValueAtPath(obj, ['items', '1']);
    
    expect(result).toBe(2);
  });
});

describe('extractFieldPath', () => {
  it('should extract path from simple RJSF field ID', () => {
    const result = extractFieldPath('root_param1');
    
    expect(result).toEqual(['param1']);
  });

  it('should extract path from nested RJSF field ID', () => {
    const result = extractFieldPath('root_param1_subparam_deepParam');
    
    expect(result).toEqual(['param1', 'subparam', 'deepParam']);
  });

  it('should handle empty segments', () => {
    const result = extractFieldPath('root_param1__subparam');
    
    expect(result).toEqual(['param1', 'subparam']);
  });
});

describe('createFieldContext', () => {
  it('should create field context from props', () => {
    const props = {
      id: 'root_param1_subparam',
      name: 'subparam',
      value: 'test value',
      schema: { type: 'string' }
    };
    
    const result = createFieldContext(props);
    
    expect(result).toEqual({
      id: 'root_param1_subparam',
      name: 'subparam',
      path: ['param1', 'subparam'],
      value: 'test value',
      schema: { type: 'string' },
      fieldName: 'subparam'
    });
  });

  it('should use last path segment as fieldName', () => {
    const props = {
      id: 'root_config_nested_deep',
      name: 'form-name',
      value: 'test',
      schema: { type: 'string' }
    };
    
    const result = createFieldContext(props);
    
    expect(result.fieldName).toBe('deep');
  });
});