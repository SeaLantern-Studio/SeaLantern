export interface SocialLinks {
  gitee?: string;
  github?: string;
  bilibili?: string;
  qq?: string;
  tiktok?: string;
  [key: string]: string | undefined;
}

export interface ContributorProfile {
  name: string;
  role: string;
  avatar: string;
  url?: string | SocialLinks;
}

export const contributors: ContributorProfile[] = [];
