export interface RootObject {
  count: number;
  next: string;
  previous: string;
  results: Result[];
}

export interface Result {
  repo_name: string;
  short_description: string;
  star_count: number;
  pull_count: number;
  repo_owner: string;
  is_automated: boolean;
  is_official: boolean;
}