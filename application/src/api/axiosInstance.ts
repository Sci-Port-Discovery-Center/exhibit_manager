import axios from "axios";

// The backend runs on the same computer as the app.
export const baseURL = "http://localhost:3030";

export const axiosInstance = axios.create({
  baseURL: baseURL,
});
