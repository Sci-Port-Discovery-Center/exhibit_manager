type Timestamp = { date: string, time: string}

export type Note = { id: string, submitter: string, timestamp: Timestamp; message: string };

export type Sponsorship = {
  sponsorName: string;
  startDate: string;
  endDate: string;
};

export type Exhibit = {
  id: string;
  name: string;
  cluster: string;
  location: string;
  description: string;
  status: "Operational" | "Needs Repair" | "Out of Service";
  part_ids: Array<string>;
  notes: Array<Note>;
  image_url: string | undefined;
  sponsorship?: Sponsorship;
};

export type Part = {
  id: string;
  name: string;
  link: string;
  exhibit_ids: Array<string>;
  notes: Array<Note>;
};

export type created_at = { date: string; time: string };
export type submitter_name = { first: string; last: string };

export type Jotform = {
  id: string;
  submitter_name: submitter_name;
  created_at: created_at;
  location: string;
  exhibit_name: string;
  description: string;
  priority_level: string;
  department: string;
  status: string;
}
