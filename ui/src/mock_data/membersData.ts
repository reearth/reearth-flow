import { Role } from "@flow/types";

export function generateTeam(numberOfMembers: number) {
  const team = [];
  for (let i = 0; i < numberOfMembers; i++) {
    team.push({
      id: i.toString(),
      name: `Member ${i}`,
      // status: i % 2 === 0 ? "online" : "offline",
      role: (i % 2 === 0 ? "writer" : "admin") as Role,
    });
  }
  return team;
}

// export const members = [
//   {
//     id: "1234",
//     name: "Kyle W",
//   },
//   {
//     id: "5678",
//     name: "John D",
//   },
//   {
//     id: "91011",
//     name: "Jane S",
//   },
//   {
//     id: "121314",
//     name: "Alice B",
//   },
//   {
//     id: "151617",
//     name: "Bob C",
//   },
//   {
//     id: "181920",
//     name: "Charlie D",
//   },
//   {
//     id: "212223",
//     name: "David E",
//   },
//   {
//     id: "242526",
//     name: "Eve F",
//   },
//   {
//     id: "272829",
//     name: "Frank G",
//   },
//   {
//     id: "303132",
//     name: "Grace H",
//   },
//   {
//     id: "333435",
//     name: "Hank I",
//   },
//   {
//     id: "363738",
//     name: "Ivy J",
//   },
//   {
//     id: "394041",
//     name: "Jack K",
//   },
//   {
//     id: "424344",
//     name: "Kate L",
//   },
//   {
//     id: "454647",
//     name: "Liam M",
//   },
//   {
//     id: "484950",
//     name: "Mia N",
//   },
//   {
//     id: "515253",
//     name: "Nate O",
//   },
//   {
//     id: "545556",
//     name: "Olive P",
//   },
//   {
//     id: "575859",
//     name: "Perry Q",
//   },
//   {
//     id: "606162",
//     name: "Quinn R",
//   },
//   {
//     id: "636465",
//     name: "Ruth S",
//   },
//   {
//     id: "666768",
//     name: "Sam T",
//   },
//   {
//     id: "697071",
//     name: "Tom U",
//   },
//   {
//     id: "727374",
//     name: "Uma V",
//   },
//   {
//     id: "757677",
//     name: "Vince W",
//   },
//   {
//     id: "787980",
//     name: "Wendy X",
//   },
//   {
//     id: "808182",
//     name: "Xander Y",
//   },
//   {
//     id: "838485",
//     name: "Yara Z",
//   },
//   {
//     id: "868788",
//     name: "Zane A",
//   },
//   {
//     id: "899091",
//     name: "Abe B",
//   },
//   {
//     id: "929394",
//     name: "Bea C",
//   },
//   {
//     id: "959697",
//     name: "Cal D",
//   },
//   {
//     id: "9899100",
//     name: "Dee E",
//   },
//   {
//     id: "101102103",
//     name: "Eli F",
//   },
//   {
//     id: "104105106",
//     name: "Fay G",
//   },
// ];
