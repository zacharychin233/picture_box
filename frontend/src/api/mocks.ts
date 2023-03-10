import { ListResponse, Resolve } from "./models";
import m from "../statics/images/m.webp";
import origin from "../statics/images/origin.jpg";
import s from "../statics/images/s.webp";
import xs from "../statics/images/xs.webp";

export const partitionsOk = ["default", "avatar", "cover"];

const resolves: Resolve[] = [];

const length = 54;
for (let i = 0; i < length; i++) {
  let json = `{"middle": "${m}", "origin": "${origin}", "s": "${s}", "${i}": "${xs}", "hello": "${xs}", "hel": "${xs}"}`;
  let obj = JSON.parse(json);
  resolves.push(obj);
}

export const listPicturesOk = (current: number): ListResponse<Resolve> => {
  let list: Resolve[] = [];
  let start = (current - 1) * 10;
  let end = start + 10;
  resolves.forEach((value, index) => {
    if (index >= start && index < end) {
      list.push(value);
    }
    if (index > end) {
      return;
    }
  });

  return {
    list: list,
    pagination: {
      current: current,
      page_size: 10,
      total: length,
    },
  };
};
