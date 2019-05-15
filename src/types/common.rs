// Monero Swap Rust Library
// Written in 2019 by
//   h4sh3d <h4sh3d@truelevel.io>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//

use super::RelativeLocktime;

#[derive(Debug, Clone)]
pub struct Params {
    pub t_0: RelativeLocktime,
    pub t_1: RelativeLocktime,
}

impl Params {
    pub fn new(t_0: RelativeLocktime, t_1: RelativeLocktime) -> Params {
        Params {
            t_0,
            t_1,
        }
    }
}
