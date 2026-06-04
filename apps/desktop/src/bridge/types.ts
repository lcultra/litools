export type SearchResultAction = {
  id: string;
  label: string;
};

export type SearchResult = {
  id: string;
  title: string;
  subtitle?: string | null;
  provider: string;
  score: number;
  actions: SearchResultAction[];
};
