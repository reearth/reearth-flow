// WIP: might use as base, or scrap. But keeping here.
// Needs rework on Canvas and useDnd to support this code.
// This only works for undo/redoing adding nodes (dnd). Breaks changing node position.
// Canvas state needs to be more thoughtfully planned out before implementing this.

import { useState } from "react";

export default <T>(initialState: T | undefined) => {
  const [past, setPast] = useState<T[]>([]);
  const [present, setPresent] = useState<T | undefined>(initialState);
  const [future, setFuture] = useState<T[]>([]);

  const undo = () => {
    if (past.length === 0) return;

    const newPast = [...past];
    const newPresent = newPast.pop();

    setPast(newPast);
    setFuture([...future, ...(present !== undefined ? [present] : [])]);
    setPresent(newPresent);
  };

  const redo = () => {
    if (future.length === 0) return;

    const newFuture = [...future];
    const newPresent = newFuture.pop();

    setPast([...past, ...(present !== undefined ? [present] : [])]);
    setFuture(newFuture);
    setPresent(newPresent);
  };

  const updatePresent = (newState: T | undefined) => {
    console.log("HI IM UPDATING PRESENT", newState);
    setPast([...past, ...(present !== undefined ? [present] : [])]);
    setPresent(newState);
    setFuture([]);
  };

  return { state: present, undo, redo, updatePresent };
};

// [{id: "1"}, {id: "2"}, {id: "3"}]

// future = []
// past == [{id: "1"}, {id: "2"}]
// present = {id: "3"}

// undo
// future = [{id: "3"}]
// past == [{id: "1"}, {id: "2"}]
// present = {id: "2"}
