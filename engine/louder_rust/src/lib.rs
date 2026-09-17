mod test_linkage;
mod louder;

#[unsafe(no_mangle)]
extern "system" fn test_linkage() -> i32 {
  test_linkage::test_linkage()
}